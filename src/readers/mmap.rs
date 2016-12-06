use std::slice;

pub fn open(path: String) -> Result<slice::Iter> {
    let input_mmap = try!(Mmap::open_path(opts.input, memmap::Protection::Read)
        .chain_err(|| "Failed to open input file as a memory map"));
    info!("Opened file with mmap: {}", path);
    let input_slice = unsafe { input_mmap.as_slice() };
    Ok(input_slice.iter())
}
