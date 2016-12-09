use std::io;
use std::io::Write;
use std::io::BufWriter;
use std::sync::Mutex;
use std::iter;

use jobsteal;

use errors::*;
use kmer_length::KmerLength;
use sort::sort;
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
            readers::mmap::open(input).map(|it| Box::new(it.map(Ok)) as Box<Iterator<Item = Result<u8>>>)
        } else {
            readers::file::open(input).map(|it| Box::new(it) as Box<Iterator<Item = Result<u8>>>)
        };
        file.chain_err(|| "Failed to open input file")
            .unwrap_or_else(|e| Box::new(iter::once(Err(e)) as Box<Iterator<Item = Result<u8>>>))
    });

    // TODO: multiple parser types
    let inputs = inputs.map(parsers::multifasta::SectionReader::new);

    // TODO: maybe a segment based implementation?
    let counts = Mutex::new(vec![]);
    let error = Mutex::new(None);
    job_pool.scope(|scope| {
        for input in inputs {
            scope.submit(move || {
                let kmers = get_kmers::Kmers::new(input, opts.kmer_len.clone());
                let items = kmers.map(|k| {
                    match k {
                        Err(e) => {
                            error.lock().unwrap() = e;
                            // None stops as iter is fused
                            None
                        },
                        Ok(k) => {
                            Some((k, 1))
                        }
                    }
                }).fuse();
                for item in items {
                    counts.lock().push(item);
                }
            });
        }
    });
    let error = try!(error.into_inner().chain_err(|| "Who knew it could go this wrong: error mutex poisoned"));
    if let Some(e) = error {
        return Err(e);
    }
    info!("Done counting {} k-mers", counts.len());

    let counts = try!(counts.into_inner().chain_err(|| "A k-mer counting thread panicked"));
    if opts.only_presence {
        sort(counts.as_mut_slice(), |_, _, _| {})
    } else {
        sort(counts.as_mut_slice(), |_, value, other| *value += other)
    };
    info!("Done sorting k-mers");

    let stdout = io::stdout();
    output_counts::output(job_pool, stdout.lock(), counts, opts.kmer_len, opts.min_count);
    info!("Done!");
    Ok(())
}
