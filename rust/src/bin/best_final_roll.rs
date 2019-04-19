extern crate yahtzeevalue;
use yahtzeevalue::*;
use yahtzeevalue::constants::*;

fn main() {
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
