use std::io;
use std::io::Write;
use std::io::BufWriter;
use std::sync::Mutex;
use std::error::Error as ErrorTrait;
use std::iter;

use jobsteal;

use errors::*;
use kmer_length::KmerLength;
use sort::sort;
use error_str::ErrorStr;
use get_kmers;
use output_counts;
use kmer_tree;

use readers;
use parsers;

/// The list of options for the runner
pub struct Options {
    pub inputs: Vec<String>,
    pub kmer_len: KmerLength,
    pub min_count: u16,
    pub only_presence: bool,
    pub threads: usize,
    pub mmap: bool,
    /// Sorted from highest depth to lowest depth
    pub join_methods: Vec<kmer_tree::JoinMethod>,
}

pub fn run(opts: Options) -> Result<()> {
    let job_pool = jobsteal::make_pool(opts.threads).unwrap();

    let inputs = opts.inputs.into_iter().map(|input| {
        let file = if opts.mmap {
            readers::mmap::open(input)
                .map(|it| Box::new(it.map(Ok)) as Box<Iterator<Item = Result<u8>> + Send + Sync>)
        } else {
            readers::file::open(input).map(|it| Box::new(it) as Box<Iterator<Item = Result<u8>> + Send + Sync>)
        };
        file.chain_err(|| "Failed to open input file")
            .unwrap_or_else(|e| Box::new(iter::once(Err(e))) as Box<Iterator<Item = Result<u8>> + Send + Sync>)
    });

    // TODO: multiple parser types
    let inputs = inputs.map(parsers::multifasta::SectionReader::new);

    let error = Mutex::new(None);
    let input_counts = Mutex::new(Ok(Vec::new()));
    job_pool.scope(|scope| {
        for input in inputs {
            scope.submit(move || {
                let section_counts = Ok(Vec::new());
                for section in input {
                    let kmers =
                        section.and_then(|section| {
                                get_kmers::Kmers::new(section, opts.kmer_len.clone())
                            })
                            .and_then(|kmer_iter| {
                                kmer_iter.map(|r| r.map(|n| Some((n, 1))))
                                    .collect::<Result<Vec<_>>>()
                            })
                            .map(|v| {
                                kmer_tree::Node::Leaf(kmer_tree::Leaf {
                                    counts: v,
                                    sorted: false,
                                })
                            });
                    match kmers {
                        Err(e) => {
                            section_counts = Err(e);
                            break;
                        }
                        Ok(vec) => {
                            section_counts.map(|list| list.push(vec));
                        }
                    }
                }
                let section_counts = section_counts.map(kmer_tree::Node::Branch);
                let input_counts = input_counts.lock().unwrap();
                match section_counts {
                    Err(e) => *input_counts = Err(e),
                    Ok(vec) => {
                        input_counts.map(|list| list.push(vec));
                    }
                }
            });
        }
    });
    let counts = try!(input_counts.lock()
        .map_err(|e| ErrorStr::new(e.description())) // e is not Sync
        .chain_err(|| "A k-mer counting thread panicked, poisoning the output mutex"));
    let mut counts = try!(counts.chain_err(|| "Encountered an error during k-mer counting"));
    info!("Done counting {} k-mers", counts.len());

    let all_counts = None;
    job_pool.scope(|scope| {
        if opts.only_presence {
            all_counts = Some(kmer_tree::Node::Branch(counts).consolidate(scope, opts.join_methods, &|_, _, _| {}).counts);
        } else {
            all_counts = Some(kmer_tree::Node::Branch(counts).consolidate(scope, opts.join_methods, &|_, value, other| *value += other).counts);
        };
    });
    let counts = all_counts.unwrap();
    info!("Done consolidating {} k-mers", counts.len());

    let stdout = io::stdout();
    output_counts::output(job_pool,
                          stdout,
                          counts,
                          opts.kmer_len,
                          opts.min_count);
    info!("Done!");
    Ok(())
}
