use std::fmt;
use std::mem;

mod constants;
use constants::*;

mod outcome;
use outcome::*;

mod state;
use state::*;

enum Action {
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
fn actions<F: FnMut(Action, u32, u32)>(state: State, o: Outcome, mut f: F) {
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
        f(Action::Side(d), state.with_side(d).with_score(new_score).encode(), s + bonus);
    }
    possible_scores(o, state, |comb, s| {
        if state.has_comb(comb) {
            return;
        }
        f(Action::Combination(comb), state.with_comb(comb).encode(), s);
    });
}

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

fn best_final_roll() {
    for i in 0..18 {
        let s = State::decode(0x3FFFF & !(1 << i));
        let mut best_action = Action::Side(SIDES);
        let mut best_points = 0;
        let mut best_outcome = None;
        for o in outcomes() {
            actions(s, o, |action, _next_state, points| {
                if points > best_points {
                    best_action = action;
                    best_points = points;
                    best_outcome = Some(o);
                }
            });
        }
        println!("{} {:?} {:?} {} {:?}", i, s, best_action, best_points, best_outcome);
    }
}

fn print_reachable() {
    let mut reachable = vec![0u8; (1 + BONUS_LIMIT as usize) << 18];
    reachable[0] = 1;
    let mut skipped = 0;
    for i in 0..reachable.len() {
        if reachable[i] == 0 {
            skipped += 1;
            continue;
        }
        let s = State::decode(i as u32);
        for o in outcomes() {
            actions(s, o, |_action, next_state, _points| {
                let next_state = next_state as usize;
                assert!(next_state > i);
                reachable[next_state] = 1;
            });
        }
    }
    println!("Skipped: {}", skipped);
}

fn iter_all() {
    let mut best_score = vec![0xFFFFu16; (1 + BONUS_LIMIT as usize) << 18];
    let mut best_so_far = 0;
    best_score[0] = 0;
    let mut skipped = 0;
    for i in 0..best_score.len() {
        if best_score[i] == 0xFFFF {
            skipped += 1;
            continue;
        }
        if i % 10000 == 0 {
            println!("{:7} {:3} {:?}", skipped, best_score[i], State::decode(i as u32));
        }
        let ub = best_score[i] + State::decode(i as u32).upper_bound_points() as u16;
        best_so_far = best_so_far.max(best_score[i]);
        let mut ub_correct = false;
        let s = State::decode(i as u32);
        for o in outcomes() {
            actions(s, o, |action, next_state, points| {
                let s = best_score[i] + points as u16;
                let ub2 = s + State::decode(next_state).upper_bound_points() as u16;
                if ub2 > ub {
                    println!("{:?} with {:?} {:?} => {:?}, {} + {} + {} = {} > {}",
                             State::decode(i as u32),
                             o, action,
                             State::decode(next_state),
                             best_score[i],
                             points,
                             State::decode(next_state).upper_bound_points(),
                             ub2, ub);
                }
                assert!(ub2 <= ub);
                if ub == ub2 {
                    ub_correct = true;
                }
                let next_state = next_state as usize;
                assert!(next_state > i);
                if best_score[next_state] == 0xFFFF {
                    best_score[next_state] = s;
                } else {
                    best_score[next_state] = best_score[next_state].max(s);
                }
            });
        }
        if !ub_correct {
            println!("TEST {:3} {:3} {:?}", best_score[i], ub, State::decode(i as u32));
        }
    }
    println!("Skipped: {}", skipped);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_successors() {
        assert_eq!(Outcome::empty().successors().map(|o| o.histogram).collect::<Vec<_>>(),
        vec![[1,0,0,0,0,0],[0,1,0,0,0,0],[0,0,1,0,0,0],[0,0,0,1,0,0],[0,0,0,0,1,0],[0,0,0,0,0,1]]);
    }

    #[test]
    fn simple_predecessors() {
        let o = Outcome {histogram: [0, 0, 6, 0, 0, 0]};
        assert_eq!(o.predecessors().map(|o| o.histogram).collect::<Vec<_>>(),
        vec![[0,0,5,0,0,0]]);
    }

    #[test]
    fn outcome_encode_decode() {
        for o in outcomes() {
            assert_eq!(o, Outcome::decode(o.encode()));
        }
    }

    #[test]
    fn state_encode_decode() {
        for i in 0..2228 {
            let s = State::decode(i * 1000);
            assert_eq!(s, State::decode(s.encode()));
        }
    }

    #[test]
    fn state_initial_encode() {
        assert_eq!(State::initial().encode(), 0);
    }

    #[test]
    fn state_initial_upper_bound() {
        assert_eq!(State::initial().upper_bound_points(), 580);
    }
}
