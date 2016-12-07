use std::slice;

use memmap;
use memmap::Mmap;

use errors::*;

pub fn open(path: String) -> Result<Mmap> {
    let mmap = try!(Mmap::open_path(path, memmap::Protection::Read)
        .chain_err(|| "Failed to open input file as a memory map"));
    debug!("Opened file with mmap: {}", path);
    Ok(mmap);
}
