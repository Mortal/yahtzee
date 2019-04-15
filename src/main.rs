use std::fmt;
use std::mem;

mod constants;
use constants::*;

mod outcome;
use outcome::*;

mod state;
use state::*;

mod scoring;
use scoring::*;

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
