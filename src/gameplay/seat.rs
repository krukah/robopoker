use crate::Chips;
use crate::cards::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seat {
    state: State,
    stack: Chips,
    stake: Chips,
    spent: Chips,
    /// this field is the only non-public state, but if we're
    /// client-side then we can just fill it with clones of our own private cards.
    /// with this very natural method of obfuscation,
    /// we can use Game struct as normal
    cards: Hole,
}

impl From<(Hole, Chips)> for Seat {
    fn from((cards, stack): (Hole, Chips)) -> Self {
        Self {
            cards,
            stack,
            spent: 0,
            stake: 0,
            state: State::Betting,
        }
    }
}

impl Seat {
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
    pub fn reset_stack(&mut self) {
        self.stack = crate::STACK;
    }
}

impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.state,
            format!("${:>4}", self.stack),
            self.cards
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Betting,
    Shoving,
    Folding,
}

impl State {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Betting | Self::Shoving)
    }
}

impl TryFrom<&str> for State {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "P" => Ok(State::Betting),
            "S" => Ok(State::Shoving),
            "F" => Ok(State::Folding),
            _ => Err(anyhow::anyhow!("invalid state string")),
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Betting => write!(f, "P"),
            State::Shoving => write!(f, "S"),
            State::Folding => write!(f, "F"),
        }
    }
}
