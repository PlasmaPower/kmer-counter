use std::io::Write;
use std::io::BufWriter;
use std::sync::Mutex;

use jobsteal::Pool;

use errors::*;
use kmer_length::KmerLength;
use nucleotide::Nucleotide;

pub fn output<T>(pool: Pool,
                 stream: T,
                 counts: Vec<Option<(u64, u16)>>,
                 kmer_len: KmerLength,
                 min_count: u16)
    where T: Write + Send
{
    let stream = Mutex::new(BufWriter::new(stream));
    let kmer_len = kmer_len.length() as usize;
    pool.scope(|scope| {
        for (kmer, count) in counts.into_iter().filter_map(|n| n) {
            if count < min_count {
                continue;
            }
            scope.submit(move || {
                let mut kmer_str = vec![0; kmer_len + 1];
                kmer_str[kmer_len] = b'\t';
                for i in (0..kmer_len).rev() {
                    let nucleotide = Nucleotide::from((kmer & 0xff) as u8);
                    let chr = nucleotide.as_text_byte();
                    kmer_str[i] = chr;
                    kmer = kmer >> 2;
                }
                let mut stream = stream.lock().unwrap();
                stream.write(kmer_str.as_slice())
                    .and_then(|_| stream.write(count.to_string().as_bytes()))
                    .and_then(|_| stream.write(b"\n"))
                    .chain_err(|| "Failed to write k-mer to output stream")
                    .unwrap();
            })
        }
    });
}
