#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Nucleotide {
    A,
    C,
    G,
    T,
}

impl Nucleotide {
    pub fn into_char(self) -> u8 {
        match self {
            Nucleotide::A => b'A',
            Nucleotide::C => b'C',
            Nucleotide::G => b'G',
            Nucleotide::T => b'T',
        }
    }
}
