pub struct Options {
    pub kmer_len: u8,
    pub kmer_bitmask: u64,
}

pub fn count(slice: &[u8], opts: &Options) -> Vec<(u64, u16)> {
    debug!("Starting to count section");

    let binary_iter = slice.iter().filter_map(|c| {
        match c | 0b00100000 {
            b'a' => Some(0b00),
            b'c' => Some(0b01),
            b'g' => Some(0b10),
            b't' => Some(0b11),
            _ => None,
        }
    });

    let mut buffer: u64 = 0u64;
    let mut buffer_len = 0;
    let mut kmer_list = Vec::new();
    for n in binary_iter {
        buffer = n as u64 + ((buffer << 2) & opts.kmer_bitmask);
        // this buffer_len logic should be unrolled by the compiler
        if buffer_len < opts.kmer_len {
            buffer_len += 1;
        }
        if buffer_len >= opts.kmer_len {
            kmer_list.push((buffer, 1));
        }
    }

    debug!("Done counting section");
    kmer_list
}
