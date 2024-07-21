#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Street {
    Pref,
    Flop,
    Turn,
    Rive,
    Show,
}

impl Street {
    pub fn next(&self) -> Street {
        match self {
            Street::Pref => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::Rive,
            Street::Rive => Street::Show,
            Street::Show => unreachable!("No next street after Showdown"),
        }
    }
}

impl Display for Street {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Street::Pref => write!(f, "Pre Flop"),
            Street::Flop => write!(f, "Flop"),
            Street::Turn => write!(f, "Turn"),
            Street::Rive => write!(f, "River"),
            Street::Show => write!(f, "Showdown"),
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
            street: Street::Pref,
        }
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for card in &self.cards {
            write!(f, "{}  ", card)?;
        }
        Ok(())
    }
}

use super::card::Card;
use std::fmt::{Display, Formatter, Result};
