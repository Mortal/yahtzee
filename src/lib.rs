pub mod constants;

mod outcome;
pub use outcome::{Outcome, outcomes, sub_outcomes, max_outcome_encoding};

mod state;
pub use state::State;

mod scoring;
pub use scoring::{Action, actions};

mod valuation;
pub use valuation::{compute_state_value, compute_outcome_values, compute_subset_expectations, compute_reroll_value, choose_reroll};

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
