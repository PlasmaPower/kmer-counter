use std::io;
use std::io::Read;
use std::sync::Mutex;
use std::error::Error as ErrorTrait;

use jobsteal;

use errors::*;
use kmer_length::KmerLength;
use error_string::ErrorString;
use get_kmers;
use output_counts;
use kmer_tree;

use readers;
use parsers;

/// The list of options for the runner
pub struct Options {
    pub inputs: Vec<String>,
    pub stdin: bool,
    pub kmer_len: KmerLength,
    pub min_count: u16,
    pub only_presence: bool,
    pub threads: usize,
    pub mmap: bool,
    pub join_methods: Vec<kmer_tree::JoinMethod>,
}

pub fn run(opts: Options) -> Result<()> {
    let Options {
        inputs,
        stdin,
        kmer_len,
        min_count,
        only_presence,
        threads,
        mmap,
        join_methods,
    } = opts;
    let mut job_pool = jobsteal::make_pool(threads).unwrap();

    let stdin_handle = io::stdin();

    let mut inputs = try!(inputs.into_iter()
        .map(|input| {
            if mmap {
                readers::mmap::open(input).map(|it| {
                        Box::new(it.map(Ok)) as Box<Iterator<Item = Result<u8>> + Send + Sync>
                    })
            } else {
                readers::file::open(input)
                    .map(|it| Box::new(it) as Box<Iterator<Item = Result<u8>> + Send + Sync>)
            }
        })
        .collect::<Result<Vec<_>>>());

    if stdin {
        inputs.push(Box::new(stdin_handle.bytes()
                             .map(|r| r.chain_err(|| "Failed to read from stdin")))
                    as Box<Iterator<Item = Result<u8>> + Send + Sync>);
    }

    // TODO: multiple parser types
    let inputs = inputs.into_iter().map(parsers::multifasta::SectionReader::new);

    let input_counts = Mutex::new(Ok(Vec::new()));
    job_pool.scope(|scope| {
        let input_counts_ref = &input_counts;
        for mut input in inputs {
            scope.submit(move || {
                let mut section_counts = Ok(Vec::new());
                while let Some(section) = input.next_section() {
                    let kmers =
                        section.and_then(|section| get_kmers::Kmers::new(section, kmer_len.clone()))
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
                            let _ = section_counts.as_mut().map(|list| list.push(vec));
                        }
                    }
                }
                let section_counts = section_counts.map(kmer_tree::Node::Branch);
                let mut input_counts = input_counts_ref.lock().unwrap();
                match section_counts {
                    Err(e) => *input_counts = Err(e),
                    Ok(vec) => {
                        let _ = input_counts.as_mut().map(|list| list.push(vec));
                    }
                }
            });
        }
    });
    let counts = try!(input_counts.into_inner()
                      .map_err(|e| ErrorString::new(e.description().to_string())) // e is not Sync
                      .chain_err(|| {
                          "A k-mer counting thread panicked, poisoning the output mutex"
                      }));
    let counts = try!(counts.chain_err(|| "Encountered an error during k-mer counting"));
    info!("Done counting {} k-mers", counts.len());

    let mut all_counts = None;
    let join_methods = join_methods.as_slice();
    job_pool.scope(|scope| {
        if only_presence {
            all_counts = Some(kmer_tree::Node::Branch(counts)
                .consolidate(scope, join_methods, &|_, _, _| {})
                .counts);
        } else {
            all_counts = Some(kmer_tree::Node::Branch(counts)
                .consolidate(scope, join_methods, &|_, value, other| *value += other)
                .counts);
        };
    });
    let counts = all_counts.unwrap();
    info!("Done consolidating {} k-mers", counts.len());

    let stdout = io::stdout();
    output_counts::output(job_pool, stdout, counts, kmer_len, min_count);
    info!("Done!");
    Ok(())
}
