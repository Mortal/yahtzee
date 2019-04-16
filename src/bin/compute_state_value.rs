//! Compute expected value for the optimal strategy in every state of
//! Super Yahtzee. Takes between 100 and 140 minutes to compute.
extern crate yahtzee;
extern crate byteorder;

use std::{io, fs};

use byteorder::{LittleEndian, WriteBytesExt};

use yahtzee::compute_state_value;

fn main() {
    let file = fs::File::create("state_value.tmp").expect("Could not open file");
    let state_value = compute_state_value(|i, n| {
        // 8 dots in a cluster, 32 dots in a line, BONUS_LIMIT lines
        // each line represents 2**18
        // each dot represents 2**13
        if i == 0 {
            eprintln!("Compute value of {} states", n);
        }
        if i != n && (i == 0 || i % (1 << 13) != 0) {
            return;
        }
        eprint!("{}", ".");
        if i == n || i % (1 << 16) == 0 {
            eprint!(" ");
            if i == n || i % (1 << 18) == 0 {
                eprint!("{:8}/{}\n", i, n);
            }
        }
    });
    {
        let mut writer = io::BufWriter::new(file);
        for x in state_value.iter() {
            writer.write_f64::<LittleEndian>(*x).expect("Writing failed");
        }
    }
    fs::rename("state_value.tmp", "state_value.bin").expect("Failed to rename state_value.tmp");
}
