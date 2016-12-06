use std::file::File;
use std::io::Read;
use std::io::BufReader;
use std::io::Bytes;

use errors;

pub struct FileIterator {
    reader: Bytes<BufReader>,
}

impl Iterator for FileIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.reader.next().map(|r| r.expect("Error reading input file"))
    }
}

pub fn open(path: String) -> Result<FileIterator> {
    let file = try!(File::open(path).chain_err(|| "Failed to open input file"));
    info!("Opened file: {}", path);
    let reader = BufReader::new(file);
    Ok(reader.bytes().map(|r| r.expect("")))
}
