use std::fmt;
use crate::constants::*;
use crate::outcome::*;
use crate::state::*;

pub enum Action {
    Combination(Comb),
    Side(usize),
}

impl fmt::Debug for Action {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Action::Combination(S2) => write!(fmt, "Action::Combination(S2)"),
            &Action::Combination(S22) => write!(fmt, "Action::Combination(S22)"),
            &Action::Combination(S222) => write!(fmt, "Action::Combination(S222)"),
            &Action::Combination(S3) => write!(fmt, "Action::Combination(S3)"),
            &Action::Combination(S4) => write!(fmt, "Action::Combination(S4)"),
            &Action::Combination(S33) => write!(fmt, "Action::Combination(S33)"),
            &Action::Combination(R15) => write!(fmt, "Action::Combination(R15)"),
            &Action::Combination(R26) => write!(fmt, "Action::Combination(R26)"),
            &Action::Combination(R16) => write!(fmt, "Action::Combination(R16)"),
            &Action::Combination(S23) => write!(fmt, "Action::Combination(S23)"),
            &Action::Combination(CHANCE) => write!(fmt, "Action::Combination(CHANCE)"),
            &Action::Combination(YAHTZEE) => write!(fmt, "Action::Combination(YAHTZEE)"),
            &Action::Combination(c) => write!(fmt, "Action::Combination({})", c),
            &Action::Side(s) => write!(fmt, "Action::Side({} - 1)", s + 1),
        }
    }
}

fn score_pairs<F: FnMut(Comb, u32)>(o: Outcome, f: &mut F) {
    let mut pair_sum = 0u32;
    let mut pairs = 0;
    let pair_scores = [S2, S22, S222];
    for d in (0..SIDES).rev() {
        if o.histogram[d] >= 2 {
            pair_sum += (d as u32 + 1) * 2;
            f(pair_scores[pairs], pair_sum);
            pairs += 1;
        }
    }
    for i in pairs..3 {
        f(pair_scores[i], 0);
    }
}

fn score_sum<F: FnMut(Comb, u32)>(o: Outcome, f: &mut F) {
    let mut sum = 0;
    for d in (0..SIDES).rev() {
        sum += (d as u32 + 1) * (o.histogram[d] as u32);
    }
    f(CHANCE, sum);
}

fn score_yahtzee<F: FnMut(Comb, u32)>(o: Outcome, f: &mut F) {
    let mut s6 = 0;
    for d in (0..SIDES).rev() {
        if o.histogram[d] == 6 {
            s6 = d as u32 + 1;
        }
    }
    f(YAHTZEE, if s6 > 0 { 100 + 6 * s6 } else { 0 });
}

fn score_combinations<F: FnMut(Comb, u32)>(o: Outcome, f: &mut F) {
    let mut s2 = 0;
    let mut s3 = 0;
    let mut s4 = 0;
    let mut s33 = 0;
    for d in (0..SIDES).rev() {
        if o.histogram[d] >= 4 {
            s2 = s2.max(s3);
            s3 = d as u32 + 1;
            s4 = s3;
        } else if o.histogram[d] == 3 {
            s2 = s2.max(s3);
            s33 = s33.max(s3);
            s3 = d as u32 + 1;
        } else if o.histogram[d] == 2 {
            s2 = d as u32 + 1;
        }
    }
    f(S4, 4 * s4);
    f(S3, 3 * s3);
    f(S33, if s33 > 0 { 3 * (s33 + s3) } else { 0 });
    f(S23, if s2 > 0 && s3 > 0 { s2 * 2 + s3 * 3 } else { 0 });
}

fn score_singles<F: FnMut(Comb, u32)>(o: Outcome, f: &mut F) {
    let mut singles = 0;
    for d in (0..SIDES).rev() {
        if o.histogram[d] == 1 {
            singles += 1;
        }
    }
    if singles == 6 {
        f(R15, 15);
        f(R26, 21);
        f(R16, 30);
    } else if singles == 4 && o.histogram[0] == 0 {
        f(R15, 0);
        f(R26, 21);
        f(R16, 0);
    } else if singles == 4 && o.histogram[5] == 0 {
        f(R15, 15);
        f(R26, 0);
        f(R16, 0);
    } else {
        f(R15, 0);
        f(R26, 0);
        f(R16, 0);
    }
}

fn possible_scores<F: FnMut(Comb, u32)>(o: Outcome, s: State, mut f: F) {
    let pairs = (1 << S2) | (1 << S22) | (1 << S222);
    if s.combination_mask & pairs != pairs { score_pairs(o, &mut f); }
    if !s.has_comb(CHANCE) { score_sum(o, &mut f); }
    if !s.has_comb(YAHTZEE) { score_yahtzee(o, &mut f); }
    let combs = (1 << S3) | (1 << S4) | (1 << S33) | (1 << S23);
    if s.combination_mask & combs != combs { score_combinations(o, &mut f); }
    let singles = (1 << R15) | (1 << R26) | (1 << R16);
    if s.combination_mask & singles != singles { score_singles(o, &mut f); }
}

// state:32 is score:7 sides:6 combinations:12
// score is in 0..85, so number of states is 85*2**18 = 22282240
// f(action, next_state, points)
pub fn actions<F: FnMut(Action, State, u32)>(state: State, o: Outcome, mut f: F) {
    let score = state.score;
    for d in 0..SIDES {
        if state.has_side(d) {
            continue;
        }
        let s = (d as u32 + 1) * (o.histogram[d] as u32);
        let (new_score, bonus) =
            if score < BONUS_LIMIT && score + s >= BONUS_LIMIT {
                (BONUS_LIMIT, BONUS)
            } else if score < BONUS_LIMIT {
                (score + s, 0)
            } else {
                (score, 0)
            };
        f(Action::Side(d), state.with_side(d).with_score(new_score), s + bonus);
    }
    possible_scores(o, state, |comb, s| {
        if state.has_comb(comb) {
            return;
        }
        f(Action::Combination(comb), state.with_comb(comb), s);
    });
}
