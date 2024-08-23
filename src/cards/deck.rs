use super::{card::Card, hand::Hand};

#[derive(Debug, Clone, Copy)]
pub struct Deck(Hand);

impl Deck {
    pub fn new() -> Self {
        Self(Hand::from((1 << 52) - 1))
    }

    pub fn flip(&mut self) -> Card {
        let index = self.0 .0.trailing_zeros();
        let card = Card::from(index as u8);
        self.0 .0 &= !(1 << index);
        card
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
