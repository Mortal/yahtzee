// Ideas for interaction:
// player 2 -> switch to player 2's turn
// set ! 42 -> set Yahtzee row to 42 (forced result)
// r 113456 -> roll and suggest actions
// S -> choose action 'S'
//
// Initial roll: "I would keep 56 to go for two pairs"
// Final roll: "I would take the obvious choice: ..." (i.e. the non-Chance one with highest score)
// List other actions and their expectations rounded to integers (or a couple decimals if some are close)
use std::io;
use std::io::BufRead;

extern crate yahtzee;
use yahtzee::*;
use yahtzee::constants::*;

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

    fn next<O, F: FnMut(&str) -> Option<O>>(&mut self, prompt: &str, mut parser: F) -> O {
        let mut p = prompt;
        loop {
            while None == self.peek_word() {
                println!("{}", p);
                p = "I did not understand that.";
                self.line.clear();
                self.reader.read_line(&mut self.line).unwrap();
                self.word = 0;
            }
            let r = self.next_word().unwrap();
            match parser(r) {
                Some(o) => return o,
                None => (),
            };
        }
    }
}

struct Player {
    state: u32,
    points: u32,
}

impl Player {
    fn new() -> Self {
        Player {
            state: 0,
            points: 0,
        }
    }
}

enum CommandWord {
    Roll(Outcome),
    Players,
    Player,
    Action(usize),
    Help,
    Score,
    Bonus,
}

fn parse_command_word(w: &str, game: &Game) -> Option<CommandWord> {
    match parse_outcome(w) {
        Some(o) => return Some(CommandWord::Roll(o)),
        None => (),
    };
    if w == "players" {
        return Some(CommandWord::Players);
    }
    if w == "player" {
        return Some(CommandWord::Player);
    }
    if w == "score" {
        return Some(CommandWord::Score);
    }
    if w == "bonus" {
        return Some(CommandWord::Bonus);
    }
    for (i, (_value, action, _state, _points)) in game.choices.iter().enumerate() {
        if w == action.shorthand() {
            return Some(CommandWord::Action(i));
        }
    }
    if w == "help" {
        return Some(CommandWord::Help);
    }
    return None
}

enum Command {
    Roll(Outcome),
    Players(usize),
    Player(usize),
    Action(u32, u32),
    Help,
    Score(i32),
    Bonus(i32),
}

struct Game {
    players: Vec<Player>,
    player_count: usize,
    player_index: usize,
    choices: Vec<(f64, Action, u32, u32)>,
    roll_index: usize,
    prompt: String,
}

fn parse_command<R: io::Read>(reader: &mut Tokenizer<R>, game: &mut Game) -> Command {
    match reader.next(&game.prompt, |w| parse_command_word(w, game)) {
        CommandWord::Players => Command::Players(reader.next("New player count:", |w| w.parse::<usize>().ok())),
        CommandWord::Player => Command::Player(reader.next("Whose turn is it?", |w| w.parse::<usize>().ok())),
        CommandWord::Roll(o) => Command::Roll(o),
        CommandWord::Action(i) => {
            let (_value, _action, state, points) = &game.choices[i];
            Command::Action(*state, *points)
        },
        CommandWord::Help => Command::Help,
        CommandWord::Score => Command::Score(reader.next("Points to add/subtract:", |w| w.parse::<i32>().ok())),
        CommandWord::Bonus => Command::Bonus(reader.next("Points to add/subtract:", |w| w.parse::<i32>().ok())),
    }
}

const HELP: &'static str = "\
Commands:
  <dice>      input roll, e.g. 113666
  <row>       put roll on given row, e.g. D for Two Pairs
  help        this help text
  players N   set number of players to N
  player N    switch current turn to player N
  bonus N     add N to score, counting towards bonus
  score N     add N to score without counting towards bonus
";

fn main() {
    let state_value = Store::new("state_value.bin").expect("Failed to read state value");
    let stdin = io::stdin();
    let mut reader = Tokenizer::new(stdin.lock());

    let mut outcome_value = vec![0.0; max_outcome_encoding() + 1];
    let mut reroll_value = vec![0.0; outcome_value.len()];

    let mut game = Game {
        players: vec![Player::new()],
        player_count: 1,
        player_index: 0,
        choices: Vec::new(),
        roll_index: 0,
        prompt: String::new(),
    };

    loop {
        let state = State::decode(game.players[game.player_index].state);
        let points = game.players[game.player_index].points;
        if game.roll_index == 0 {
            let player_prompt = if game.player_count > 1 { format!("P{} ", game.player_index + 1) } else { String::new() };
            game.prompt = format!("{}{:3} {} Input roll or command or 'help':", player_prompt, state.display_score(points), state);
        }
        match parse_command(&mut reader, &mut game) {
            Command::Roll(mut outcome) => {
                if game.roll_index == 0 {
                    compute_outcome_values(state, &mut |i| state_value.get(i), &mut outcome_value);
                    compute_subset_expectations(&mut outcome_value);
                    compute_reroll_value(&outcome_value, &mut reroll_value);
                    compute_subset_expectations(&mut reroll_value);
                    choose_reroll(&mut outcome, &reroll_value);
                    game.prompt = format!("I would keep {}. Input roll:", outcome);
                    game.roll_index += 1;
                } else if game.roll_index == 1 {
                    compute_outcome_values(state, &mut |i| state_value.get(i), &mut outcome_value);
                    compute_subset_expectations(&mut outcome_value);
                    choose_reroll(&mut outcome, &outcome_value);
                    game.prompt = format!("I would keep {}. Input roll:", outcome);
                    game.roll_index += 1;
                    game.choices.clear();
                } else if game.roll_index == 2 {
                    game.choices.clear();
                    actions(state, outcome, |action, next_state, action_points| {
                        let i = next_state.encode();
                        let value = state_value.get(i) + points as f64 + action_points as f64 - BONUS_LIMIT as f64;
                        game.choices.push((value, action, i, action_points));
                    });
                    game.choices.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
                    game.choices.reverse();
                    for (i, (value, action, _state, points)) in game.choices.iter().enumerate() {
                        if i == 0 {
                            println!("I would choose '{}' {} for {} points (E={:.1}). All possibilities:", action.shorthand(), action.name(), points, value);
                        }
                        println!("  {}  {:25} {:3} pts (E={:.1})", action.shorthand(), action.name(), points, value);
                    }
                    game.prompt = "Which action do you choose?".to_owned();
                }
            },
            Command::Players(n) => {
                game.player_count = n;
                while game.players.len() < game.player_count {
                    game.players.push(Player::new());
                }
                game.choices.clear();
            },
            Command::Player(i) => {
                if i >= 1 && i <= game.player_count {
                    game.player_index = i - 1;
                }
                game.choices.clear();
            },
            Command::Action(s, p) => {
                game.players[game.player_index].state = s;
                game.players[game.player_index].points += p;
                game.player_index = (game.player_index + 1) % game.player_count;
                game.choices.clear();
                game.roll_index = 0;
            },
            Command::Help => {
                println!("{}", HELP);
            },
            Command::Score(n) => {
                let p = &mut game.players[game.player_index].points;
                if n < 0 {
                    *p = p.saturating_sub((-n) as u32);
                } else {
                    *p = p.saturating_add(n as u32);
                }
            },
            Command::Bonus(n) => {
                let p = &mut game.players[game.player_index];
                let mut s = State::decode(p.state);
                if n < 0 {
                    p.points = p.points.saturating_sub((-n) as u32);
                    s.score = s.score.saturating_sub((-n) as u32);
                } else {
                    p.points = p.points.saturating_add(n as u32);
                    s.score = s.score.saturating_add(n as u32).min(BONUS_LIMIT);
                }
                p.state = s.encode();
            },
        }
    }
}
