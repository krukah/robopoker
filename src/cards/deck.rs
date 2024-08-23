use super::card::Card;
use super::hand::Hand;

/// Deck extends much of Hand functionality, with ability to remove cards from itself. Random selection via ::draw(), or sequential via ::flip().
#[derive(Debug, Clone, Copy)]
pub struct Deck(Hand);

impl Deck {
    pub fn new() -> Self {
        Self(Hand::from((1 << 52) - 1))
    }

    pub fn flip(&mut self) -> Card {
        let value = u64::from(self.0);
        let zeros = value.trailing_zeros();
        self.0 = Hand::from(value & !(1 << zeros));
        Card::from(zeros as u8)
    }

    pub fn draw(&mut self) -> Card {
        //? TODO: index should be a randomly selected bit index to distinguish from flip
        let value = u64::from(self.0);
        let index = value.trailing_zeros();
        self.0 = Hand::from(value & !(1 << index));
        Card::from(index as u8)
    }
}
