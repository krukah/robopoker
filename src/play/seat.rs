use super::Chips;
use crate::cards::hole::Hole;
use colored::Colorize;

#[derive(Debug, Clone, Copy)]
pub struct Seat {
    cards: Hole,
    spent: Chips,
    stack: Chips,
    stake: Chips,
    state: State,
}

impl Seat {
    pub fn new(stack: Chips) -> Seat {
        Seat {
            stack,
            spent: 0,
            stake: 0,
            state: State::Playing,
            cards: Hole::empty(),
        }
    }
    pub fn stack(&self) -> Chips {
        self.stack
    }
    pub fn stake(&self) -> Chips {
        self.stake
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn spent(&self) -> Chips {
        self.spent
    }
    pub fn cards(&self) -> Hole {
        self.cards
    }

    pub fn win(&mut self, win: Chips) {
        self.stack += win;
    }
    pub fn bet(&mut self, bet: &Chips) {
        self.stack -= bet;
        self.stake += bet;
        self.spent += bet;
    }
    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
    pub fn set_cards(&mut self, cards: Hole) {
        self.cards = cards;
    }
    pub fn set_stake(&mut self) {
        self.stake = 0;
    }
    pub fn set_spent(&mut self) {
        self.spent = 0;
    }
}

impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            format!("{:>4}", self.stack).green(),
            self.cards,
            self.state,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Playing,
    Shoving,
    Folding,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Playing => write!(f, "{}", "P".green()),
            State::Shoving => write!(f, "{}", "S".yellow()),
            State::Folding => write!(f, "{}", "F".red()),
        }
    }
}
