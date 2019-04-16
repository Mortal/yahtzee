extern crate rand;
extern crate byteorder;

use std::{io, fs};
use byteorder::{LittleEndian, ReadBytesExt};

extern crate yahtzee;
use yahtzee::*;
use yahtzee::constants::*;

fn random_outcome<R: rand::Rng>(rng: &mut R) -> Outcome {
    let mut outcome = Outcome::empty();
    for _ in 0..DICE_COUNT {
        let roll = rng.gen_range(0, SIDES);
        outcome.histogram[roll] += 1;
    }
    outcome
}

fn random_reroll<R: rand::Rng>(rng: &mut R, outcome: &mut Outcome) {
    let mut c = 0;
    for i in 0..SIDES {
        c += outcome.histogram[i];
    }
    for _ in (c as usize)..DICE_COUNT {
        let roll = rng.gen_range(0, SIDES);
        outcome.histogram[roll] += 1;
    }
}

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

fn test_state_value(state_value: &Vec<f64>) {
    let s1 = State::initial().with_side(0).with_score(5);
    let s2 = State::initial().with_comb(S33);
    let e1 = state_value[s1.encode() as usize] + 5.0 - BONUS_LIMIT as f64;
    let e2 = state_value[s2.encode() as usize] - BONUS_LIMIT as f64;
    println!("For 5 1's, scratching 2x3 has expectation {}, taking the 1's has expectation {}", e2, e1);
    if e2 > e1 {
        for d in 0..SIDES {
            for c in 0..DICE_COUNT + 1 {
                let score = c as u32 * (1 + d) as u32;
                let s = State::initial().with_side(d).with_score(score);
                let e = state_value[s.encode() as usize] + score as f64 - BONUS_LIMIT as f64;
                println!("Keeping {} {}'s => {}", c, d + 1, e);
            }
        }
        panic!("Wrong expectation");
    }
}

fn main() {
    let state_value = read_state_value().expect("Failed to read state value");
    test_state_value(&state_value);
    println!("Expected score: {}", state_value[0] - BONUS_LIMIT as f64);
    let mut rng = rand::thread_rng();
    let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
    let mut reroll_value = vec![0.0; outcome_value.len()];
    loop {
        let mut points = 0;
        let mut state = State {
            combination_mask: 0,
            sides_mask: 0,
            score: 0,
        };
        while !state.done() {
            let mut outcome = random_outcome(&mut rng);
            print!("{:3} {} Roll {}", state.display_score(points), state, outcome);
            compute_outcome_values(state, &state_value, &mut outcome_value);
            compute_subset_expectations(&mut outcome_value);
            compute_reroll_value(&outcome_value, &mut reroll_value);
            compute_subset_expectations(&mut reroll_value);
            choose_reroll(&mut outcome, &reroll_value);
            print!(", keep {:6}", format!("{}", outcome));
            random_reroll(&mut rng, &mut outcome);
            print!(" and reroll to {}", outcome);
            choose_reroll(&mut outcome, &outcome_value);
            print!(", keep {:6}", format!("{}", outcome));
            random_reroll(&mut rng, &mut outcome);
            print!(" and reroll to {}", outcome);

            let mut best = -1f64;
            let mut best_action = None;
            let mut best_state = None;
            let mut best_points = 0;
            let mut action_count = 0;
            actions(state, outcome, |action, next_state, points| {
                action_count += 1;
                let i = next_state.encode() as usize;
                let value = state_value[i] + points as f64;
                if value > best {
                    best = value;
                    best_action = Some(action);
                    best_state = Some(next_state);
                    best_points = points;
                }
            });
            assert!(action_count > 0);
            println!(", {} => {:3} points (exp.: {:.4})", best_action.unwrap(), best_points, points as f64 + best - BONUS_LIMIT as f64);
            state = best_state.unwrap();
            points += best_points;
        }
        println!("\n");
    }
}
