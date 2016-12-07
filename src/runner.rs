use std::io;
use std::io::Write;
use std::io::BufWriter;
use std::sync::Mutex;

use errors::*;
use count;
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
    pub input: String,
    pub kmer_len: u8,
    pub min_count: u16,
    pub only_presence: bool,
    pub threads: usize,
    // TODO:
    pub join_methods: Vec<JoinMethod>,
}

pub fn run(opts: Options) -> Result<()> {
    let job_pool = jobsteal::make_pool(opts.threads).unwrap();

    // TODO: multiple input files
    let input = try!(readers::mmap::open(opts.input).map(Box::new).or_else(|e| {
        error!("Failed to open memory map, trying regular file reading");
        error!("{}", e);
        Box::new(readers::file::open(opts.input))
    }).chain_err(|| "Failed to open input file"));
    info!("Input file opened");

    let parser = parsers::multifasta::SectionReader::new(input)

    // TODO: rewrite here to jobsteal (per input not section), parser
    let counts = run_counts::run_counts(input_slice, &count_opts);
    info!("Done counting {} k-mers", counts.len());

    let stdout = io::stdout();
    output_counts::output(job_pool, stdout.lock());
    Ok(())
}
