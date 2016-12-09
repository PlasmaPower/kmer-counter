use std::fs::File;
use std::io::Read;
use std::io::BufReader;
use std::io::Bytes;

use errors::*;

pub struct FileIterator {
    reader: Bytes<BufReader<File>>,
}

impl Iterator for FileIterator {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next().chain_err(|| "Error reading input file")
    }
}

pub fn open(path: String) -> Result<FileIterator> {
    let file = try!(File::open(path).chain_err(|| "Failed to open input file"));
    info!("Opened file: {}", path);
    let reader = BufReader::new(file);
    reader.bytes()
}
