#[derive(Debug, Clone)]
pub struct Hole {
    pub cards: Vec<Card>, // presize
}

impl Hole {
    pub fn new() -> Hole {
        Hole {
            cards: Vec::with_capacity(2),
        }
    }
}

impl Display for Hole {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} {}", self.cards[0], self.cards[1])
    }
}

use super::card::Card;
use std::fmt::{Display, Formatter, Result};
