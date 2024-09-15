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
        use rand::Rng;
        let deck = u64::from(self.0);
        let mut rng = rand::thread_rng();
        let mut card = rng.gen_range(0..64);
        while deck & (1 << card) == 0 {
            card = rng.gen_range(0..64);
        }
        self.0 = Hand::from(deck & !(1 << card));
        Card::from(card as u8)
    }
}
