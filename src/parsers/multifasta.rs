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
            let c = match c {
                Ok(c) => c,
                Err(e) => return Some(Err(e)),
            };
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
        None
    }
}

pub struct SectionReader<T> {
    file: T,
}

impl<T: Iterator<Item = Result<u8>>> SectionReader<T> {
    pub fn new(file: T) -> SectionReader<T> {
        SectionReader {
            file: file,
        }
    }

    pub fn next_section<'a>(&'a mut self) -> Option<Result<Section<'a, T>>> {
        loop {
            match self.file.next() {
                None => return None,
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(b'\n')) => break,
                Some(_) => continue,
            }
        }
        Some(Ok(Section {
            file: &mut self.file,
            done: false,
        }))
    }
}
