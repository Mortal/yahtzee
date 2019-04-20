extern crate byteorder;
extern crate memmap;

use std::fs;
use byteorder::{ByteOrder, LittleEndian};
use crate::*;
use crate::constants::*;

pub struct Store {
    mmap: memmap::Mmap,
}

impl Store {
    pub fn new(path: &str) -> Result<Store> {
        let file = fs::File::open(path)?;
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

    pub fn best_action(&self, state: u32, outcome: Outcome) -> Option<usize> {
        let mut best = -1f64;
        let mut best_action = None;
        actions(State::decode(state), outcome, |action, next_state, points| {
            let i = next_state.encode();
            let value = self.get(i) + points as f64;
            if value > best {
                best = value;
                best_action = Some(action);
            }
        });
        let best_action = match best_action {
            Some(a) => a,
            None => return None,
        };
        match best_action {
            Action::Combination(v) => Some(v),
            Action::Side(s) => Some(s + COMB_COUNT),
        }
    }

    pub fn keep_first(&self, state: u32, mut outcome: Outcome) -> u32 {
        let state = State::decode(state);
        let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
        let mut reroll_value = vec![0.0; outcome_value.len()];
        compute_outcome_values(state, &mut |i| self.get(i), &mut outcome_value);
        compute_subset_expectations(&mut outcome_value);
        compute_reroll_value(&outcome_value, &mut reroll_value);
        compute_subset_expectations(&mut reroll_value);
        choose_reroll(&mut outcome, &reroll_value);
        outcome.encode()
    }

    pub fn keep_second(&self, state: u32, mut outcome: Outcome) -> u32 {
        let state = State::decode(state);
        let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
        compute_outcome_values(state, &mut |i| self.get(i), &mut outcome_value);
        compute_subset_expectations(&mut outcome_value);
        choose_reroll(&mut outcome, &outcome_value);
        outcome.encode()
    }
}
