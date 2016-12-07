use std::marker::PhantomData;

use errors::*;
use nucleotide::Nucleotide;

pub struct Section<'a, T: 'a> {
    file: &'a mut T,
    done: bool,
}

impl<'a, T: Iterator<Item = Result<u8>>> Iterator for Section<'a, T> {
    type Item = Result<Nucleotide>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        while let Some(c) = self.file.next() {
            let c = try!(c);
            match c {
                b' ' | b'\n' | b'\t' => continue,
                b'>' => {
                    self.done = true;
                    return None;
                }
                _ => {
                    if let Some(n) = Nucleotide::from_text_byte(c) {
                        return Some(Ok(n));
                    } else {
                        warn!("Encountered invalid character in input multifasta: {}",
                              c as char);
                        continue;
                    }
                }
            }
        }
    }
}

pub struct SectionReader<'a, T: 'a> {
    file: T,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: Iterator<Item = Result<u8>>> SectionReader<'a, T> {
    pub fn new(file: T) -> SectionReader<'a, T> {
        SectionReader { file: file }
    }
}

impl<'a, T: Iterator<Item = u8>> Iterator for SectionReader<'a, T> {
    type Item = Result<Section<'a, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match try!(self.file.next()) {
                None => return None,
                Some(b'\n') => break,
                Some(_) => continue,
            }
        }
        Some(Ok(Section { file: &mut self.file }))
    }
}
