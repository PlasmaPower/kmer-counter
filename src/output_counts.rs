use std::io::Write;

use jobsteal::Pool;

use errors::*;

pub fn output(pool: Pool, stream: Write) {
    let stream = Mutex::new(BufWriter::new(stream));
    let kmer_len = opts.kmer_len.length() as usize;
    pool.scope(|scope| {
        for (kmer, count) in counts.filter_map(|n| n) {
            if count < opts.min_count {
                continue;
            }
            scope.submit(move || {
                let mut kmer_str = vec![0; kmer_len + 1];
                kmer_str[kmer_len] = b'\t';
                for i in (0..kmer_len).rev() {
                    let nucleotide = Nucleotide::from(kmer & 0xff);
                    let chr = nucleotide.to_text_byte();
                    kmer_str[i] = chr;
                    kmer = kmer >> 2;
                }
                let mut stream = stream.lock();
                stream.write(kmer_str.as_slice())
                    .and_then(|_| stream.write(count.to_string().as_bytes()))
                    .and_then(|_| stream.write(b"\n"))
                    .chain_err(|| "Failed to write k-mer to output stream")
                    .unwrap();
            })
        }
    });
}
