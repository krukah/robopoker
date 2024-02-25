#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Street {
    Pre,
    Flop,
    Turn,
    River,
}

impl Display for Street {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Street::Pre => write!(f, "Pre Flop"),
            Street::Flop => write!(f, "Flop"),
            Street::Turn => write!(f, "Turn"),
            Street::River => write!(f, "River"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub cards: Vec<Card>, // presize
    pub street: Street,
}

impl Board {
    pub fn new() -> Board {
        Board {
            cards: Vec::with_capacity(5),
            street: Street::Pre,
        }
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }
}

use super::card::Card;
use std::fmt::{Display, Formatter, Result};
