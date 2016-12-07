#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Nucleotide {
    A,
    C,
    G,
    T,
}

impl Nucleotide {
    pub fn into_text_byte(self) -> u8 {
        match self {
            Nucleotide::A => b'A',
            Nucleotide::C => b'C',
            Nucleotide::G => b'G',
            Nucleotide::T => b'T',
        }
    }

    pub fn from_text_byte(c: u8) -> Option<Nucleotide> {
        match c {
            b'a' | b'A' => Some(Nucleotide::A),
            b'c' | b'C' => Some(Nucleotide::C),
            b'g' | b'G' => Some(Nucleotide::G),
            b't' | b'T' => Some(Nucleotide::T),
            _ => None,
        }
    }
}

impl Into<u8> for Nucleotide {
    /// Result guarenteed to fit within 2 bits.
    /// Also guarenteed to have same sorting.
    /// This function compiles down to a move.
    #[inline]
    pub fn into(self) -> u8 {
        match self {
            Nucleotide::A => 0,
            Nucleotide::C => 1,
            Nucleotide::G => 2,
            Nucleotide::T => 3,
        }
    }
}

impl From<u8> for Nucleotide {
    /// This function compiles down to a move.
    #[inline]
    pub fn into(self) -> u8 {
        match self {
            0 => Nucleotide::A,
            1 => Nucleotide::C,
            2 => Nucleotide::G,
            3 => Nucleotide::T,
        }
    }
}