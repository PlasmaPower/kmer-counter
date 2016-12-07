use nucleotide::Nucleotide;
use kmer_length::KmerLength;

pub fn<T: Iterator<Nucleotide>> count(input: T, kmer_len: KmerLength) -> Vec<(u64, u16)> {
    debug!("Starting to count section");

    let mut buffer: u64 = 0u64;
    let mut buffer_len = 0;
    let mut kmer_list = Vec::new();
    for n in input {
        buffer = n.into() + ((buffer << 2) & kmer_len.bitmask());
        // this buffer_len logic should be unrolled by the compiler
        if buffer_len < kmer_len.length() {
            buffer_len += 1;
        }
        if buffer_len >= kmer_len.length() {
            kmer_list.push((buffer, 1));
        }
    }

    debug!("Done counting section");
    kmer_list
}
