// Ideas for interaction:
// player 2 -> switch to player 2's turn
// set ! 42 -> set Yahtzee row to 42 (forced result)
// r 113456 -> roll and suggest actions
// S -> choose action 'S'
//
// Initial roll: "I would keep 56 to go for two pairs"
// Final roll: "I would take the obvious choice: ..." (i.e. the non-Chance one with highest score)
// List other actions and their expectations rounded to integers (or a couple decimals if some are close)
extern crate byteorder;

use std::{io, fs};
use std::io::BufRead;
use byteorder::{LittleEndian, ReadBytesExt};

extern crate yahtzee;
use yahtzee::*;
use yahtzee::constants::*;

fn read_state_value() -> io::Result<Vec<f64>> {
    let file = fs::File::open("state_value.bin")?;
    let size = file.metadata()?.len() as usize;
    let mut reader = io::BufReader::new(file);
    let mut state_value = vec![0f64; size / 8];
    for x in state_value.iter_mut() {
        *x = reader.read_f64::<LittleEndian>()?;
    }
    Ok(state_value)
}

fn parse_outcome(line: &str) -> Option<Outcome> {
    if line.len() != DICE_COUNT {
        return None;
    }
    let mut outcome = Outcome::empty();
    for c in line.chars() {
        if c < '1' {
            return None;
        }
        let v = c as usize - '1' as usize;
        if v >= SIDES {
            return None;
        }
        outcome.histogram[v] += 1;
    }
    Some(outcome)
}

struct Tokenizer<R: io::Read> {
    reader: io::BufReader<R>,
    line: String,
    word: usize,
}

impl <R: io::Read> Tokenizer<R> {
    fn new(inner: R) -> Self {
        Tokenizer {
            reader: io::BufReader::new(inner),
            line: String::new(),
            word: 0,
        }
    }

    fn peek_word(&self) -> Option<&str> {
        self.line.split_whitespace().nth(self.word)
    }

    fn next_word(&mut self) -> Option<&str> {
        let res = self.line.split_whitespace().nth(self.word);
        self.word += 1;
        res
    }

    fn next<O, F: FnMut(&str) -> Option<O>>(&'a mut self, prompt: &str, mut parser: F) -> Option<O> {
        loop {
            while None == self.peek_word() {
                println!("{}", prompt);
                self.line.clear();
                self.reader.read_line(&mut self.line).unwrap();
                self.word = 0;
            }
            let r = self.next_word();
        }
    }
}

fn main() {
    let state_value = read_state_value().expect("Failed to read state value");
    let stdin = io::stdin();
    let mut reader = Tokenizer::new(stdin.lock());

    let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
    let mut reroll_value = vec![0.0; outcome_value.len()];

    let mut points = 0;
    let mut state = State {
        combination_mask: 0,
        sides_mask: 0,
        score: 0,
    };
    while !state.done() {
        let word = reader.next(format!("{:3} {} Input roll:", state.display_score(points), state), |w| parse_outcome(w));
        let mut outcome = match parse_outcome(word) {
            Some(o) => o,
            None => {
                println!("Couldn't understand your roll {:?}", line);
                continue;
            }
        };

        compute_outcome_values(state, &state_value, &mut outcome_value);
        compute_subset_expectations(&mut outcome_value);
        compute_reroll_value(&outcome_value, &mut reroll_value);
        compute_subset_expectations(&mut reroll_value);
        choose_reroll(&mut outcome, &reroll_value);
        println!("I would keep {}. Input roll:", outcome);

        line_buf.clear();
        reader.read_line(&mut line_buf).unwrap();
        let line = line_buf.trim();
        let mut outcome = match parse_outcome(line) {
            Some(o) => o,
            None => {
                println!("Couldn't understand your roll {:?}", line);
                continue;
            }
        };

        choose_reroll(&mut outcome, &outcome_value);
        println!("I would keep {}. Input roll:", outcome);

        line_buf.clear();
        reader.read_line(&mut line_buf).unwrap();
        let line = line_buf.trim();
        let outcome = match parse_outcome(line) {
            Some(o) => o,
            None => {
                println!("Couldn't understand your roll {:?}", line);
                continue;
            }
        };

        let mut choices = Vec::new();
        println!("Choose an action:");

        actions(state, outcome, |action, next_state, points| {
            choices.push((next_state, points));
            let i = next_state.encode() as usize;
            let value = state_value[i] + points as f64 - BONUS_LIMIT as f64;
            println!("{}. {} => {:3} points (exp.: {:.4})", choices.len(), action, points, value);
        });

        line_buf.clear();
        reader.read_line(&mut line_buf).unwrap();
        let line = line_buf.trim();
        let i = match line.parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                println!("Couldn't understand your choice");
                continue;
            }
        };
        if i < 1 || i > choices.len() {
            println!("Index out of range");
            continue;
        }
        let (next_state, p) = choices[i - 1];
        state = next_state;
        points += p;
    }
}
