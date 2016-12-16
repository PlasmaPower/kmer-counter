use kmer_length::KmerLength;

#[derive(PartialEq, Eq, Clone)]
pub enum JoinMethod {
    Concat,
    Join,
    Sort,
}

pub trait Collector {
    fn push(&mut self, count: (u64, u16));
    fn finalize(&mut self) {}
    fn subdivide(&mut self, join_method: JoinMethod) -> Box<&mut Collector>;
}

pub struct OutputCollector<T> {
    out: BufWriter<T>,
    kmer_len: KmerLength,
    min_len: u16,
}

impl<T: Write> OutputCollector<T> {
    pub fn new(out: T, kmer_len: KmerLength, min_len: u16) {
        OutputCollector {
            out: BufWriter::new(out),
            kmer_len: kmer_len,
            min_len: min_len,
        }
    }
}

impl<T: Write> Collector for OutputCollector<T> {
    fn push(&mut self, (kmer, count): (u64, u16)) {
        if count < self.min_count {
            continue;
        }
        let mut kmer_str = vec![0; kmer_len + 1];
        kmer_str[kmer_len] = b'\t';
        for i in (0..kmer_len).rev() {
            let nucleotide = Nucleotide::from_lower_bits(kmer as u8);
            let chr = nucleotide.as_text_byte();
            kmer_str[i] = chr;
            kmer = kmer >> 2;
        }
        stream.write(kmer_str.as_slice())
            .and_then(|_| stream.write(count.to_string().as_bytes()))
            .and_then(|_| stream.write(b"\n"))
            .chain_err(|| "Failed to write k-mer to output stream")
            .unwrap();
    }
}
