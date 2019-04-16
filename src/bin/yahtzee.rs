extern crate rand;

use std::mem;

extern crate yahtzee;
use yahtzee::*;
use yahtzee::constants::*;

fn max_outcome_encoding() -> usize {
    outcomes().map(|o| o.encode()).max().unwrap() as usize
}

fn compute_outcome_values(state: State, state_value: &Vec<f64>, outcome_value: &mut Vec<f64>) {
    for o in outcomes() {
        let mut best = 0f64;
        actions(state, o, |_action, next_state, points| {
            let i = next_state.combination_mask as usize;
            let value = state_value[i] + points as f64;
            best = best.max(value);
        });
        outcome_value[o.encode() as usize] = best;
    }
}

fn compute_subset_expectations(outcome_value: &mut Vec<f64>) {
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
}

fn each_subset_help<F: FnMut(Outcome)>(mut outcome: Outcome, f: &mut F, mut i: usize) {
    while i < SIDES && outcome.histogram[i] == 0 {
        i += 1;
    }
    if i == SIDES {
        f(outcome);
    } else {
        for j in 0..outcome.histogram[i] + 1 {
            outcome.histogram[i] = j;
            each_subset_help(outcome, f, i + 1);
        }
    }
}

fn each_subset<F: FnMut(Outcome)>(outcome: Outcome, mut f: F) {
    each_subset_help(outcome, &mut f, 0);
}

fn choose_reroll(outcome: &mut Outcome, reroll_value: &Vec<f64>) {
    let mut best = reroll_value[outcome.encode() as usize];
    each_subset(*outcome, |o| {
        let value = reroll_value[o.encode() as usize];
        if value > best {
            best = value;
            *outcome = o;
        }
    });
}

fn compute_best_subset_values(best_subset_value: &mut Vec<f64>) {
    // Compute best expected value when keeping a subset
    for n in 1..(DICE_COUNT + 1) {
        for o in sub_outcomes(n) {
            let i = o.encode() as usize;
            for p in o.predecessors() {
                best_subset_value[i] = best_subset_value[i].max(best_subset_value[p.encode() as usize]);
            }
        }
    }
}

fn compute_reroll_value(outcome_value: &Vec<f64>, best_subset_value: &mut Vec<f64>) {
    best_subset_value.resize(outcome_value.len(), 0.0);
    for i in 0..outcome_value.len() {
        best_subset_value[i] = outcome_value[i];
    }
    compute_best_subset_values(best_subset_value);
}

fn expectation_over_outcomes(outcome_value: &Vec<f64>) -> f64 {
    let mut numerator = 0.0;
    let mut denominator = 0;
    for o in outcomes() {
        let i = o.encode() as usize;
        let m = o.multiplicity();
        numerator += m as f64 * outcome_value[i];
        denominator += m;
    }
    numerator / denominator as f64
}

fn compute_state_value() -> Vec<f64> {
    let mut state_value = vec![0.0; 0x1000];
    let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
    let mut best_subset_value = Vec::new();
    for i in (0..0x0fff).rev() {
        let s = State {
            combination_mask: i as u16,
            sides_mask: 0x3F,
            score: BONUS_LIMIT,
        };
        compute_outcome_values(s, &state_value, &mut outcome_value);
        compute_subset_expectations(&mut outcome_value);

        for _ in 0..REROLL_COUNT {
            compute_reroll_value(&outcome_value, &mut best_subset_value);
            compute_subset_expectations(&mut best_subset_value);
            mem::swap(&mut outcome_value, &mut best_subset_value);
        }

        state_value[i] = expectation_over_outcomes(&outcome_value);
    }
    state_value
}

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

fn main() {
    let state_value = compute_state_value();
    let mut rng = rand::thread_rng();
    let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
    let mut reroll_value = vec![0.0; outcome_value.len()];
    loop {
        let mut points = 0;
        let mut state = State {
            combination_mask: 0,
            sides_mask: 0x3F,
            score: BONUS_LIMIT,
        };
        while state.combination_mask != 0xFFF {
            let mut outcome = random_outcome(&mut rng);
            print!("{:3} {} Roll {}", points, state, outcome);
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
                let i = next_state.combination_mask as usize;
                let value = state_value[i] + points as f64;
                if value > best {
                    best = value;
                    best_action = Some(action);
                    best_state = Some(next_state);
                    best_points = points;
                }
            });
            assert!(action_count > 0);
            println!(", {:3} points (exp.: {:.4})", best_points, points as f64 + best);
            state = best_state.unwrap();
            points += best_points;
        }
        println!("");
    }
}
