use std::file::File;

pub struct Section<'a, T> {
    file: &'a mut T,
    done: bool,
}

impl<T: Iterator<Item=u8> Iterator for Section<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        while let Some(c) = try!(self.file.next()) {
            match c {
                b'a' | b'A' => 0b00,
                b'c' | b'C' => 0b01,
                b'g' | b'G' => 0b10,
                b't' | b'T' => 0b11,
                b' ' | b'\n' | b'\t' => continue,
                b'>' => {
                    self.done = true;
                    return None;
                }
                _ => {
                    warn!("Encountered invalid character in input multifasta: {}", c as char);
                    continue;
                }
            }
        }
    }
}

pub struct SectionReader<T> {
    file: T,
}

impl<T: Iterator<Item=u8>> SectionReader<T> {
    pub fn new(file: T) -> SectionReader<T> {
        while let Some(c) = file.next {
            match c {
                b'\n' => continue,
                b' ' => continue,
                _ => {
                    if c != b'>' {
                        warn!("Multifasta file did not start with a '>' but a '{}'", c as char);
                    }
                    break;
                }
            }
        }
        SectionReader {
            file: file,
        }
    }
}

impl<T: Iterator<Item=u8>> Iterator for SectionReader<T> {
    type Item = &mut Section<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.file.next() {
                None => return None,
                Some(b'\n') => break,
                Some(_) => continue,
            }
        }
        Some(Section { file: &mut self.file })
    }
}
