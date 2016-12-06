use std::io;
use std::io::Write;
use std::io::BufWriter;

use errors::*;
use run_counts;

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
    // TODO:
    pub join_methods: Vec<JoinMethod>,
}

pub fn run(opts: Options) -> Result<()> {
    let count_opts = run_counts::Options {
        kmer_len: opts.kmer_len,
        only_presence: opts.only_presence,
    };
    let min_count = opts.min_count;

    let input_mmap = try!(Mmap::open_path(opts.input, memmap::Protection::Read)
        .chain_err(|| "Failed to open input file as a memory map"));
    let input_slice = unsafe { input_mmap.as_slice() };
    info!("Input file opened");

    let counts = run_counts::run_counts(input_slice, &count_opts)
        .into_iter()
        .filter_map(|n| n)
        .filter(|&(_, c)| c >= min_count)
        .collect::<Vec<_>>();

    info!("Done counting {} k-mers", counts.len());
    let kmer_len = opts.kmer_len as usize;
    let mut output_list = Vec::new();
    // TODO: combine with filters, switch to jobsteal
    counts.par_iter()
        .map(|&(mut kmer, count)| {
            let mut kmer_str = vec![b'0'; kmer_len + 1];
            kmer_str[kmer_len] = b'\t';
            for i in (0..kmer_len).rev() {
                let chr = match kmer & 0b11 {
                    0b00 => b'A',
                    0b01 => b'C',
                    0b10 => b'G',
                    0b11 => b'T',
                    _ => unreachable!(),
                };
                kmer_str[i] = chr;
                kmer = kmer >> 2;
            }
            (kmer_str, count)
        })
        .collect_into(&mut output_list);
    info!("Done transforming k-mers to text form");

    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    for (kmer_str, count) in output_list {
        try!(stdout.write(kmer_str.as_slice())
            .and_then(|_| stdout.write(count.to_string().as_bytes()))
            .and_then(|_| stdout.write(b"\n"))
            .chain_err(|| "Failed to write k-mer to stdout"));
    }
    info!("Done outputting");
    Ok(())
}
