use std::mem;

use crate::*;
use crate::constants::*;

pub fn compute_outcome_values(state: State, state_value: &Vec<f64>, outcome_value: &mut Vec<f64>) {
    for o in outcomes() {
        let mut best = 0f64;
        actions(state, o, |_action, next_state, points| {
            let i = next_state.encode() as usize;
            let value = state_value[i] + points as f64;
            best = best.max(value);
        });
        outcome_value[o.encode() as usize] = best;
    }
}

pub fn compute_subset_expectations(outcome_value: &mut Vec<f64>) {
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

pub fn choose_reroll(outcome: &mut Outcome, reroll_value: &Vec<f64>) {
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

pub fn compute_reroll_value(outcome_value: &Vec<f64>, best_subset_value: &mut Vec<f64>) {
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

pub fn compute_state_value<F: FnMut(usize, usize)>(mut pi: F) -> Vec<f64> {
    let states = (1 + BONUS_LIMIT as usize) << 18;
    let mut state_value = vec![11111111111.0; states];
    let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
    let mut best_subset_value = Vec::new();
    for i in (0..states).rev() {
        let s = State::decode(i as u32);
        compute_outcome_values(s, &state_value, &mut outcome_value);
        compute_subset_expectations(&mut outcome_value);

        for _ in 0..REROLL_COUNT {
            compute_reroll_value(&outcome_value, &mut best_subset_value);
            compute_subset_expectations(&mut best_subset_value);
            mem::swap(&mut outcome_value, &mut best_subset_value);
        }

        state_value[i] = expectation_over_outcomes(&outcome_value);
        if state_value[i] > 1000.0 {
            panic!("State {} got expectation {}", s, state_value[i]);
        }
        if (states - i) % (1 << 10) == 0 {
            pi(states - i, states);
        }
    }
    state_value
}
