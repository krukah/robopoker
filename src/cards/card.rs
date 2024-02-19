use super::{rank::Rank, suit::Suit};

pub struct Card {
    rank: Rank,
    suit: Suit,
}
impl Card {
    pub fn to_int(&self) -> u8 {
        (self.rank as u8) * 4 + (self.suit as u8)
    }
}
impl From<u8> for Card {
    fn from(n: u8) -> Self {
        Card {
            rank: Rank::from(n / 4),
            suit: Suit::from(n % 4),
        }
    }
}
