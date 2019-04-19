extern crate byteorder;
extern crate memmap;

use std::{fs, io, result};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct Error(&'static str);

pub type Result<T> = result::Result<T, Error>;

pub struct Store {
    mmap: memmap::Mmap,
}

impl From<io::Error> for Error {
    fn from(_e: io::Error) -> Error {
        Error("io::Error")
    }
}

impl Store {
    pub fn new() -> Result<Store> {
        let file = fs::File::open("state_value.bin")?;
        let mmap = unsafe { memmap::Mmap::map(&file)? };
        Ok(Store {
            mmap: mmap,
        })
    }

    pub fn len(&self) -> u32 {
        let l = self.mmap.len() / 8;
        assert!(l <= std::u32::MAX as usize);
        l as u32
    }

    pub fn get(&self, s: u32) -> f64 {
        assert!(s < self.len());
        let i = 8 * s as usize;
        LittleEndian::read_f64(&self.mmap[i..i+8])
    }
}
