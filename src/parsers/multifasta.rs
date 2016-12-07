use std::file::File;

use errors;
use nucleotide::Nucleotide;

pub struct Section<'a, T> {
    file: &'a mut T,
    done: bool,
}

impl<T: Iterator<Item = Result<u8>>> Iterator for Section<T> {
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

pub struct SectionReader<T> {
    file: T,
}

impl<T: Iterator<Item = Result<u8>>> SectionReader<T> {
    pub fn new(file: T) -> SectionReader<T> {
        SectionReader { file: file }
    }
}

impl<T: Iterator<Item = u8>> Iterator for SectionReader<T> {
    type Item = Result<&mut Section<T>>;

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
