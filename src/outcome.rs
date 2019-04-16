use std::fmt;

use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outcome {
    pub histogram: [u8; SIDES],
}

pub struct OutcomePredecessorIterator<'a> {
    outcome: &'a Outcome,
    next: usize,
}

pub struct OutcomeSuccessorIterator<'a> {
    outcome: &'a Outcome,
    next: usize,
}

impl Outcome {
    pub fn empty() -> Self {
        Outcome { histogram: [0; SIDES] }
    }

    pub fn encode(&self) -> u32 {
        let mut r = 0;
        let mut a = 1;
        for d in 0..SIDES {
            r += a * self.histogram[d] as u32;
            a *= (DICE_COUNT + 1) as u32;
        }
        r
    }

    pub fn decode(mut v: u32) -> Self {
        let mut histogram = [0u8; SIDES];
        for d in 0..SIDES {
            histogram[d] = (v % (DICE_COUNT + 1) as u32) as u8;
            v /= (DICE_COUNT + 1) as u32;
        }
        Outcome { histogram: histogram }
    }

    pub fn multiplicity(&self) -> usize {
        let mut fac = [0; DICE_COUNT];
        fac[0] = 1;
        for i in 1..fac.len() {
            fac[i] = fac[i - 1] * (i + 1);
        }
        let mut res = fac[fac.len() - 1];
        for i in 0..SIDES {
            if self.histogram[i] > 0 {
                res /= fac[self.histogram[i] as usize - 1];
            }
        }
        res
    }

    pub fn predecessors(&self) -> OutcomePredecessorIterator {
        OutcomePredecessorIterator {
            outcome: self,
            next: 0,
        }
    }

    pub fn successors(&self) -> OutcomeSuccessorIterator {
        OutcomeSuccessorIterator {
            outcome: self,
            next: 0,
        }
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.histogram.len() {
            for _ in 0..self.histogram[i] {
                write!(f, "{}", i + 1)?;
            }
        }
        Ok(())
    }
}

pub fn max_outcome_encoding() -> usize {
    outcomes().map(|o| o.encode()).max().unwrap() as usize
}

impl <'a> Iterator for OutcomePredecessorIterator<'a> {
    type Item = Outcome;

    fn next(&mut self) -> Option<Outcome> {
        while self.next < SIDES && self.outcome.histogram[self.next] == 0 {
            self.next += 1;
        }
        if self.next == SIDES {
            return None;
        }
        let mut histogram = self.outcome.histogram;
        histogram[self.next] -= 1;
        self.next += 1;
        Some(Outcome { histogram: histogram })
    }
}

impl <'a> Iterator for OutcomeSuccessorIterator<'a> {
    type Item = Outcome;

    fn next(&mut self) -> Option<Outcome> {
        if self.next == SIDES {
            return None;
        }
        let mut histogram = self.outcome.histogram;
        histogram[self.next] += 1;
        self.next += 1;
        Some(Outcome { histogram: histogram })
    }
}

pub struct OutcomeIterator {
    histogram: [u8; SIDES],
}

impl OutcomeIterator {
    fn new(dice_count: u8) -> Self {
        let mut h = [0; SIDES];
        h[0] = dice_count;
        OutcomeIterator { histogram: h }
    }
}

pub fn outcomes() -> OutcomeIterator {
    OutcomeIterator::new(DICE_COUNT as u8)
}

pub fn sub_outcomes(dice_count: usize) -> OutcomeIterator {
    OutcomeIterator::new(dice_count as u8)
}

impl Iterator for OutcomeIterator {
    type Item = Outcome;

    fn next(&mut self) -> Option<Outcome> {
        let mut i = 0;
        while i < SIDES && self.histogram[i] == 0 {
            i += 1;
        }
        if i == SIDES {
            return None;
        }
        let result = Outcome { histogram: self.histogram };
        if i + 1 == SIDES {
            self.histogram[i] = 0;
        } else {
            self.histogram[i + 1] += 1;
            let v = self.histogram[i] - 1;
            self.histogram[i] = 0;
            self.histogram[0] = v;
        }
        Some(result)
    }
}
