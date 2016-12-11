use std::fs::File;
use std::io::Read;
use std::io::BufReader;
use std::io::Bytes;

use errors::*;

pub struct Iter {
    reader: Bytes<BufReader<File>>,
}

impl Iterator for Iter {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next().map(|r| r.chain_err(|| "Error reading input file"))
    }
}

pub fn open(path: String) -> Result<Iter> {
    let file = try!(File::open(path).chain_err(|| "Failed to open input file"));
    let reader = BufReader::new(file);
    Ok(Iter { reader: reader.bytes() })
}
