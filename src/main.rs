use std::fmt;

const SIDES: usize = 6;
const DICE_COUNT: usize = 6;
const BONUS_LIMIT: u32 = 4 * (1 + 2 + 3 + 4 + 5 + 6);
const BONUS: u32 = 50;

#[derive(Debug, Clone, Copy)]
struct Outcome {
    histogram: [u8; SIDES],
}

impl Outcome {
    fn permutations(&self) -> usize {
        let mut fac = [0; SIDES];
        fac[0] = 1;
        for i in 1..SIDES {
            fac[i] = fac[i - 1] * (i + 1);
        }
        let mut res = fac[SIDES - 1];
        for i in 0..SIDES {
            if self.histogram[i] > 0 {
                res /= fac[self.histogram[i] as usize - 1];
            }
        }
        res
    }
}

struct OutcomeIterator {
    histogram: [u8; SIDES],
}

impl OutcomeIterator {
    fn new() -> Self {
        let mut h = [0; SIDES];
        h[0] = DICE_COUNT as u8;
        OutcomeIterator { histogram: h }
    }
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

type Comb = usize;

const S2: Comb = 0;
const S22: Comb = 1;
const S222: Comb = 2;
const S3: Comb = 3;
const S4: Comb = 4;
const S33: Comb = 5;
const R15: Comb = 6;
const R26: Comb = 7;
const R16: Comb = 8;
const House: Comb = 9;
const Chance: Comb = 10;
const Yahtzee: Comb = 11;

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
            &Action::Combination(House) => write!(fmt, "Action::Combination(House)"),
            &Action::Combination(Chance) => write!(fmt, "Action::Combination(Chance)"),
            &Action::Combination(Yahtzee) => write!(fmt, "Action::Combination(Yahtzee)"),
            &Action::Combination(c) => write!(fmt, "Action::Combination({})", c),
            &Action::Side(s) => write!(fmt, "Action::Side({} - 1)", s + 1),
        }
    }
}

fn possible_scores<F: FnMut(Comb, u32)>(o: Outcome, mut f: F) {
    // 1 pair
    // 2 pairs
    // 3 pairs
    // 3 of a kind
    // 4 of a kind
    // 2x 3 of a kind
    // 1-5 (15)
    // 2-6 (20)
    // 1-6 (30)
    // House
    // Chance
    // Yatzy (100 + dice sum)
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
    let mut s2 = 0;
    let mut s3 = 0;
    let mut s4 = 0;
    let mut s33 = 0;
    let mut singles = 0;
    let mut sum = 0;
    let mut s6 = 0;
    for d in (0..SIDES).rev() {
        sum += (d as u32 + 1) * (o.histogram[d] as u32);
        if o.histogram[d] == 6 {
            s6 = d as u32 + 1;
        }
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
        } else if o.histogram[d] == 1 {
            singles += 1;
        }
    }
    f(S4, 4 * s4);
    f(S3, 3 * s3);
    f(S2, 2 * s2);
    f(S33, if s33 > 0 { 3 * (s33 + s3) } else { 0 });
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
    f(House, if s2 > 0 && s3 > 0 { s2 * 2 + s3 * 3 } else { 0 });
    f(Chance, sum);
    f(Yahtzee, if s6 > 0 { 100 + 6 * s6 } else { 0 });
}

struct State(u32);

impl State {
    fn combination_mask(&self) -> u32 {
        self.0 & 0xFFF
    }

    fn sides_mask(&self) -> u32 {
        (self.0 >> 12) & 0x3F
    }

    fn score(&self) -> u32 {
        self.0 >> 18
    }

    fn has_side(&self, side: usize) -> bool {
        self.sides_mask() & (1 << side) != 0
    }

    fn with_side(&self, side: usize) -> State {
        State(self.0 | (1 << (side + 12)))
    }

    fn has_comb(&self, comb: Comb) -> bool {
        self.combination_mask() & (1 << comb) != 0
    }

    fn with_comb(&self, comb: Comb) -> State {
        State(self.0 | (1 << comb))
    }

    fn with_score(&self, score: u32) -> State {
        State(self.0 & (0x3FFFF) | (score << 18))
    }

    fn upper_bound_points(&self) -> u32 {
        let mut ub = 0;
        let mut score = self.score();
        for d in 0..SIDES {
            if self.has_side(d) { continue; }
            let s = (d as u32 + 1) * (DICE_COUNT as u32);
            if score < BONUS_LIMIT && score + s >= BONUS_LIMIT {
                score = BONUS_LIMIT;
                ub += BONUS;
            } else {
                score += s;
            }
            ub += s;
        }
        if !self.has_comb(S2) { ub += 2 * SIDES as u32; }
        if !self.has_comb(S22) { ub += 4 * SIDES as u32 - 2; }
        if !self.has_comb(S222) { ub += 6 * SIDES as u32 - 6; }
        if !self.has_comb(S3) { ub += 3 * SIDES as u32; }
        if !self.has_comb(S4) { ub += 4 * SIDES as u32; }
        if !self.has_comb(S33) { ub += 6 * SIDES as u32 - 3; }
        if !self.has_comb(R15) { ub += 1 + 2 + 3 + 4 + 5; }
        if !self.has_comb(R26) { ub += 2 + 3 + 4 + 5 + 6; }
        if !self.has_comb(R16) { ub += 1 + 2 + 3 + 4 + 5 + 6; }
        if !self.has_comb(House) { ub += 5 * SIDES as u32 - 2; }
        if !self.has_comb(Chance) { ub += DICE_COUNT as u32 * SIDES as u32; }
        if !self.has_comb(Yahtzee) { ub += 100 + DICE_COUNT as u32 * SIDES as u32; }
        // 405+126+50 = 581
        ub
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "State(({:2} << 18) | (0x{:02x} << 12) | 0x{:03x})",
            self.score(), self.sides_mask(), self.combination_mask())
    }
}

// state:32 is score:7 sides:6 combinations:12
// score is in 0..85, so number of states is 85*2**18 = 22282240
// f(action, next_state, points)
fn actions<F: FnMut(Action, u32, u32)>(state: u32, o: Outcome, mut f: F) {
    let state = State(state);
    let score = state.score();
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
        f(Action::Side(d), state.with_side(d).with_score(new_score).0, s + bonus);
    }
    possible_scores(o, |comb, s| {
        if state.has_comb(comb) {
            return;
        }
        f(Action::Combination(comb), state.with_comb(comb).0, s);
    });
}

fn main() {
    let mut reachable = vec![0u8; (1 + BONUS_LIMIT as usize) << 18];
    reachable[0] = 1;
    let mut skipped = 0;
    for i in 0..reachable.len() {
        if reachable[i] == 0 {
            skipped += 1;
            continue;
        }
        for o in OutcomeIterator::new() {
            actions(i as u32, o, |action, next_state, points| {
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
            println!("{:7} {:3} {:?}", skipped, best_score[i], State(i as u32));
        }
        let ub = best_score[i] + State(i as u32).upper_bound_points() as u16;
        best_so_far = best_so_far.max(best_score[i]);
        let mut ub_correct = false;
        for o in OutcomeIterator::new() {
            actions(i as u32, o, |action, next_state, points| {
                let s = best_score[i] + points as u16;
                let ub2 = s + State(next_state).upper_bound_points() as u16;
                if ub2 > ub {
                    println!("{:?} with {:?} {:?} => {:?}, {} + {} + {} = {} > {}",
                             State(i as u32),
                             o, action,
                             State(next_state),
                             best_score[i],
                             points,
                             State(next_state).upper_bound_points(),
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
            println!("TEST {:3} {:3} {:?}", best_score[i], ub, State(i as u32));
        }
    }
    println!("Skipped: {}", skipped);
}
