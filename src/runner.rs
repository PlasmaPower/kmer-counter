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

use readers;
use parsers;

pub enum JoinMethod {
    Concat,
    Sort,
}

pub struct Options {
    pub inputs: Vec<String>,
    pub kmer_len: KmerLength,
    pub min_count: u16,
    pub only_presence: bool,
    pub threads: usize,
    // TODO:
    pub join_methods: Vec<JoinMethod>,
}

pub fn run(opts: Options) -> Result<()> {
    let job_pool = jobsteal::make_pool(opts.threads).unwrap();

    let mmap_store = Vec::new();
    let inputs = opts.inputs.into_iter().map(|input| {
        let file = if opts.mmap {
            readers::mmap::open(input).map(|mmap| {
                mmap_store.push(mmap);
                Box::new(unsafe { mmap.as_slice() }.iter())
            });
        } else {
            readers::file::open(input).map(Box::new)
        };
        file.chain_err(|| "Failed to open input file")
            .unwrap_or_else(|e| iter::once(Err(e)));
    });

    // TODO: multiple parser types
    let parsers = inputs.map(parsers::multifasta::SectionReader::new);

    // TODO: maybe a segment based implementation?
    let counts = Mutex::new(vec![]);
    let error = Mutex::new(None);
    job_pool.scope(|scope| {
        for input in inputs {
            scope.submit(move || {
                let kmers = get_kmers::Kmers::new(input, opts.kmer_len.clone());
                let items = kmers.filter_map(|k| {
                    // return None == stop
                    match k {
                        Err(e) => {
                            error.lock().unwrap() = e;
                            None
                        },
                        Ok(k) => {
                            Some(Some((k, 1)))
                        }
                    }
                }).fold();
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
