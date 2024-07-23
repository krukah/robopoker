use super::{card::Card, hand::Hand};

#[derive(Debug, Clone, Copy)]
pub struct Deck(Hand);

impl Deck {
    pub fn new() -> Self {
        Self(Hand::from((1 << 52) - 1))
    }

    pub fn draw(&mut self) -> Card {
        self.0.draw()
    }
}

// u64 isomorphism
impl From<u64> for Deck {
    fn from(n: u64) -> Self {
        Self(Hand::from(n))
    }
}
impl From<Deck> for u64 {
    fn from(deck: Deck) -> u64 {
        u64::from(deck.0)
    }
}
