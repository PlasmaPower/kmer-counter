use errors::*;
use nucleotide::Nucleotide;
use kmer_length::KmerLength;

pub struct Kmers<T> {
    input: T,
    kmer_len: KmerLength,
    buffer: u64,
}

impl<T: Iterator<Item = Result<Nucleotide>>> Kmers<T> {
    pub fn new(input: T, kmer_len: KmerLength) -> Option<Result<Kmers<T>>> {
        let buffer = 0u64;
        for _ in 0..kmer_len.length() {
            match input.next() {
                Some(Ok(n)) => {
                    let n: u8 = n.into();
                    buffer = n as u64 + (buffer << 2)
                },
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
        Some(Ok(Kmers {
            input: input,
            kmer_len: kmer_len,
            buffer: buffer,
        }))
    }
}

impl<T: Iterator<Item = Result<Nucleotide>>> Iterator for Kmers<T> {
    type Item = Result<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next().map(|n| {
            let n: u8 = try!(n).into();
            self.buffer = n as u64 + ((self.buffer << 2) & self.kmer_len.bitmask());
            Ok(self.buffer)
        })
    }
}
