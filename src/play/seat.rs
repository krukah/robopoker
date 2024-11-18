use crate::cards::hole::Hole;
use crate::Chips;
use colored::Colorize;

#[derive(Debug, Clone, Copy)]
pub struct Seat {
    cards: Hole,
    state: State,
    stack: Chips,
    stake: Chips,
    spent: Chips,
}

impl From<Chips> for Seat {
    fn from(stack: Chips) -> Self {
        Self::new(stack)
    }
}

impl Seat {
    fn new(stack: Chips) -> Seat {
        Seat {
            stack,
            spent: 0,
            stake: 0,
            state: State::Betting,
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
    pub fn bet(&mut self, bet: Chips) {
        self.stack -= bet;
        self.stake += bet;
        self.spent += bet;
    }
    pub fn reset_state(&mut self, state: State) {
        self.state = state;
    }
    pub fn reset_cards(&mut self, cards: Hole) {
        self.cards = cards;
    }
    pub fn reset_stake(&mut self) {
        self.stake = 0;
    }
    pub fn reset_spent(&mut self) {
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
    Betting,
    Shoving,
    Folding,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Betting => write!(f, "{}", "P".green()),
            State::Shoving => write!(f, "{}", "S".yellow()),
            State::Folding => write!(f, "{}", "F".red()),
        }
    }
}
