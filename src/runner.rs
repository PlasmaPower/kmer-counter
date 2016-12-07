use std::io;
use std::io::Write;
use std::io::BufWriter;
use std::sync::Mutex;

use errors::*;
use count;

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
    let pool = jobsteal::make_pool(opts.threads).unwrap();

    // TODO: use files API
    let input_mmap = try!(Mmap::open_path(opts.input, memmap::Protection::Read)
        .chain_err(|| "Failed to open input file as a memory map"));
    let input_slice = unsafe { input_mmap.as_slice() };
    info!("Input file opened");

    // TODO: rewrite here to jobsteal
    let counts = run_counts::run_counts(input_slice, &count_opts);

    info!("Done counting {} k-mers", counts.len());
    let kmer_len = opts.kmer_len as usize;
    let stdout = io::stdout();
    let mut stdout = Mutex::new(BufWriter::new(stdout.lock()));
    pool.scope(|scope| {
        for (kmer, count) in counts.filter_map(|n| n) {
            if count < opts.min_count {
                continue;
            }
            scope.submit(move || {
                let mut kmer_str = vec![b'0'; kmer_len + 1];
                kmer_str[kmer_len] = b'\t';
                for i in (0..kmer_len).rev() {
                    let nucleotide = Nucleotide::from(kmer & 0xff);
                    let chr = nucleotide.to_text_byte();
                    kmer_str[i] = chr;
                    kmer = kmer >> 2;
                }
                let mut stdout = stdout.lock();
                stdout.write(kmer_str.as_slice())
                    .and_then(|_| stdout.write(count.to_string().as_bytes()))
                    .and_then(|_| stdout.write(b"\n"))
                    .chain_err(|| "Failed to write k-mer to stdout")
                    .unwrap();
            })
        }
    });
    for (kmer_str, count) in output_list {
        try!(stdout.write(kmer_str.as_slice())
            .and_then(|_| stdout.write(count.to_string().as_bytes()))
            .and_then(|_| stdout.write(b"\n"))
            .chain_err(|| "Failed to write k-mer to stdout"));
    }
    info!("Done outputting");
    Ok(())
}
