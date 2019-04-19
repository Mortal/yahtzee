extern crate byteorder;

use std::{io, fs};
use std::io::BufRead;
use byteorder::{LittleEndian, ReadBytesExt};

extern crate yahtzeevalue;
use yahtzeevalue::*;
use yahtzeevalue::constants::*;

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

fn precompute_state(state: State) -> bool {
    let t = state.turn_count();
    t % 2 == 0 && t != 8 && t != 10
}

fn main() {
    let all_state_value = read_state_value().expect("Failed to read state value");
    let mut state_value_map = Vec::new();
    for (i, &v) in all_state_value.iter().enumerate() {
        if precompute_state(State::decode(i as u32)) {
            state_value_map.push((i, v));
        }
    }
}
