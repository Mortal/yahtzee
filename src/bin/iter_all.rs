extern crate yahtzee;
use yahtzee::*;
use yahtzee::constants::*;

fn main() {
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
                let ub2 = s + next_state.upper_bound_points() as u16;
                if ub2 > ub {
                    println!("{:?} with {:?} {:?} => {:?}, {} + {} + {} = {} > {}",
                             State::decode(i as u32),
                             o, action,
                             next_state,
                             best_score[i],
                             points,
                             next_state.upper_bound_points(),
                             ub2, ub);
                }
                assert!(ub2 <= ub);
                if ub == ub2 {
                    ub_correct = true;
                }
                let next_state = next_state.encode() as usize;
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
