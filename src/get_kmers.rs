use errors::*;
use nucleotide::Nucleotide;
use kmer_length::KmerLength;

pub struct Kmers<T> {
    input: T,
    kmer_len: KmerLength,
    buffer: u64,
}

impl<T: Iterator<Item = Result<Nucleotide>>> Kmers<T> {
    pub fn new(mut input: T, kmer_len: KmerLength) -> Result<Kmers<T>> {
        let mut buffer = 0u64;
        for _ in 0..(kmer_len.length() - 1) {
            match input.next() {
                Some(Ok(n)) => {
                    let n: u8 = n.into();
                    buffer = n as u64 + (buffer << 2)
                }
                Some(Err(e)) => return Err(e),
                // The iterator will just return nothing:
                None => break,
            }
        }
        Ok(Kmers {
            input: input,
            kmer_len: kmer_len,
            buffer: buffer,
        })
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
