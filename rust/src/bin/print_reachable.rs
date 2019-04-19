extern crate yahtzeevalue;
use yahtzeevalue::*;
use yahtzeevalue::constants::*;

fn main() {
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
                let next_state = next_state.encode() as usize;
                assert!(next_state > i);
                reachable[next_state] = 1;
            });
        }
    }
    println!("Skipped: {}", skipped);
}
