use std::slice;

use memmap;
use memmap::Mmap;

use errors::*;

pub struct Iter<'a> {
    mmap: Mmap,
    iter: slice::Iter<'a, u8>,
}

impl<'a> Iter<'a> {
    fn new(mmap: Mmap) -> Iter<'a> {
        Iter {
            mmap: mmap,
            iter: unsafe { mmap.as_slice() }.iter(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.iter.next().map(|n| *n)
    }
}

pub fn open<'a>(path: String) -> Result<Iter<'a>> {
    let mmap = try!(Mmap::open_path(path, memmap::Protection::Read)
        .chain_err(|| "Failed to open input file as a memory map"));
    debug!("Opened file with mmap: {}", path);
    Ok(Iter::new(mmap))
}
