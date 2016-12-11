use memmap;
use memmap::Mmap;

use errors::*;

// Mmap is there simply for the Drop impl
#[allow(dead_code)]
pub struct Iter {
    mmap: Mmap,
    ptr: *const u8,
    end: *const u8,
}

impl Iter {
    fn new(mmap: Mmap) -> Iter {
        let ptr = mmap.ptr();
        let len = mmap.len();
        if len > ::std::isize::MAX as usize {
            panic!("Tried to mmap file bigger than the maximum isize value");
        }
        let len = len as isize;
        let end = unsafe { ptr.offset(len) };
        Iter {
            mmap: mmap,
            ptr: ptr,
            end: end,
        }
    }
}

impl Iterator for Iter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.ptr == self.end {
            None
        } else {
            unsafe {
                let old = self.ptr;
                self.ptr = self.ptr.offset(1);
                Some(*old)
            }
        }
    }
}

unsafe impl Sync for Iter {}
unsafe impl Send for Iter {}

pub fn open(path: String) -> Result<Iter> {
    let mmap = try!(Mmap::open_path(path, memmap::Protection::Read)
        .chain_err(|| "Failed to open input file as a memory map"));
    Ok(Iter::new(mmap))
}
