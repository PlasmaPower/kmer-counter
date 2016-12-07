use nucleotide::Nucleotide;
use kmer_length::KmerLength;

pub struct Kmers<T> {
    input: T,
    kmer_len: KmerLength,
    buffer: u64,
}

impl<T: Iterator<Result<Nucleotide>>> Kmers<T> {
    pub fn new(input: T, kmer_len: KmerLength) -> Option<Result<Kmers<T>>> {
        let buffer = 0u64;
        for _ in 0..kmer_len.length() {
            match input.next() {
                Some(Ok(n)) => buffer = n.into() + (buffer << 2),
                r @ _ => return r,
            }
        }
        Some(Kmers {
            input: input,
            kmer_len: kmer_len,
            buffer: buffer,
        })
    }
}

impl<T: Iterator<Result<Nucleotide>>> Iterator for Kmers<T> {
    type Item = Result<(u64, u16)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next().map(|n| {
            self.buffer = try!(n).into() + ((self.buffer << 2) & kmer_len.bitmask());
            self.buffer
        })
    }
}
