use std::io;
use std::io::Write;
use std::io::BufWriter;
use std::sync::Mutex;
use std::iter;

use errors::*;
use get_kmers;
use output_counts;

use readers;
use parsers;

use memmap;
use memmap::Mmap;
use rayon::prelude::*;

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

    let inputs = opts.inputs.into_iter().map(|input| {
        let file = if opts.mmap {
            Box::new(readers::mmap::open(input))
        } else {
            Box::new(readers::file::open(input))
        };
        file.chain_err(|| "Failed to open input file")
            .unwrap_or_else(|e| iter::single(Err(e)));
    });

    // TODO: multiple parser types
    let parsers = inputs.map(parsers::multifasta::SectionReader::new);

    // TODO: maybe a segment based implementation?
    let counts = Mutex::new(vec![]);
    let error = Mutex::new(None);
    pool.scope(|scope| {
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
    output_counts::output(job_pool, stdout.lock());
    info!("Done!");
    Ok(())
}
