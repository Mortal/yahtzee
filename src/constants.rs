pub const SIDES: usize = 6;
pub const DICE_COUNT: usize = 6;
pub const BONUS_LIMIT: u32 = 4 * (1 + 2 + 3 + 4 + 5 + 6);
pub const BONUS: u32 = 50;
pub const REROLL_COUNT: usize = 2;

pub type Comb = usize;

pub const S2: Comb = 0;
pub const S22: Comb = 1;
pub const S222: Comb = 2;
pub const S3: Comb = 3;
pub const S4: Comb = 4;
pub const S33: Comb = 5;
pub const R15: Comb = 6;
pub const R26: Comb = 7;
pub const R16: Comb = 8;
pub const S23: Comb = 9;
pub const CHANCE: Comb = 10;
pub const YAHTZEE: Comb = 11;
