extern crate byteorder;
extern crate yahtzee;

use std::{io, fs};
use byteorder::{LittleEndian, ReadBytesExt};

use yahtzee::*;

fn read_state_value() -> io::Result<Vec<f64>> {
    let file = fs::File::open("state_value.bin")?;
    let size = file.metadata()?.len() as usize;
    let mut reader = io::BufReader::new(file);
    let mut state_value = vec![0f64; size / 8];
    for x in state_value.iter_mut() {
        *x = reader.read_f64::<LittleEndian>()?;
    }
    Ok(state_value)
}

fn main() {
    let state_value = read_state_value().expect("Failed to read state value");
    for i in 0..state_value.len() {
        let s = State::decode(i as u32);
        println!("{:20} {:08x} {} {} {:?}", format!("{}", state_value[i]), i, s.score, s, s);
    }
}
