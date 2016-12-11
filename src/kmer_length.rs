#[derive(Clone, Copy)]
pub struct KmerLength {
    length: u8,
    bitmask: u64,
}

impl KmerLength {
    pub fn new(length: u8) -> KmerLength {
        let bitmask = if length < 32 {
            (1 << (2 * (length as u64))) - 1
        } else {
            // We don't want it to overflow
            ::std::u64::MAX
        };
        KmerLength {
            length: length,
            bitmask: bitmask,
        }
    }

    #[inline]
    pub fn length(&self) -> u8 {
        self.length
    }

    #[inline]
    pub fn bitmask(&self) -> u64 {
        self.bitmask
    }
}
