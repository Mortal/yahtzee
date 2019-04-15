use std::mem;

extern crate yahtzee;
use yahtzee::*;
use yahtzee::constants::*;

fn max_outcome_encoding() -> usize {
    outcomes().map(|o| o.encode()).max().unwrap() as usize
}

fn main() {
    let mut state_value = vec![0.0; 0x1000];
    let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
    let mut best_subset_value = vec![0.0; max_outcome_encoding() + 1];
    for i in (0..0x0fff).rev() {
        let s = State { combination_mask: i as u16, sides_mask: 0x3F, score: BONUS_LIMIT };
        // Compute value of each outcome
        for o in outcomes() {
            let mut best = 0f64;
            actions(s, o, |_action, next_state, points| {
                best = best.max(state_value[State::decode(next_state).combination_mask as usize] + points as f64);
            });
            outcome_value[o.encode() as usize] = best;
        }

        for _ in 0..REROLL_COUNT {
            // Compute expected value when keeping a subset
            for n in (1..DICE_COUNT).rev() {
                for o in sub_outcomes(n) {
                    let i = o.encode() as usize;
                    outcome_value[i] = 0.0;
                    for s in o.successors() {
                        outcome_value[i] += outcome_value[s.encode() as usize];
                    }
                    outcome_value[i] /= SIDES as f64;
                }
            }
            outcome_value[0] = 0.0;
            for s in Outcome::empty().successors() {
                let i = s.encode() as usize;
                outcome_value[0] += outcome_value[i];
            }
            outcome_value[0] /= SIDES as f64;

            // Compute best expected value when keeping a subset
            best_subset_value[0] = outcome_value[0];
            for n in 1..(DICE_COUNT + 1) {
                for o in sub_outcomes(n) {
                    let i = o.encode() as usize;
                    best_subset_value[i] = outcome_value[i];
                    for p in o.predecessors() {
                        best_subset_value[i] = best_subset_value[i].max(best_subset_value[p.encode() as usize]);
                    }
                }
            }
            mem::swap(&mut outcome_value, &mut best_subset_value);
        }

        let mut numerator = 0.0;
        let mut denominator = 0;
        for o in outcomes() {
            let i = o.encode() as usize;
            let m = o.multiplicity();
            numerator += m as f64 * outcome_value[i];
            denominator += m;
        }
        state_value[i] = numerator / denominator as f64;
        println!("{} {}", s, state_value[i]);
    }
}
