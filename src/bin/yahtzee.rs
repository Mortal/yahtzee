extern crate rand;

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
