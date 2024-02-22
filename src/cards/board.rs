#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Street {
    Pre,
    Flop,
    Turn,
    River,
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
