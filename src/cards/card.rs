#[derive(Debug, Clone, Copy)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    pub fn to_int(&self) -> u8 {
        (self.rank as u8) * 4 + (self.suit as u8)
    }
    pub fn to_bits(&self) -> u64 {
        1 << u8::from(*self)
    }
}
// u8 isomorphism
impl From<Card> for u8 {
    fn from(c: Card) -> u8 {
        (c.rank as u8) * 4 + (c.suit as u8)
    }
}
impl From<u8> for Card {
    fn from(n: u8) -> Self {
        Self {
            rank: Rank::from(n / 4),
            suit: Suit::from(n % 4),
        }
    }
}
// u64 isomorphism
impl From<Card> for u64 {
    fn from(c: Card) -> u64 {
        1 << u8::from(c)
    }
}
impl From<u64> for Card {
    fn from(n: u64) -> Self {
        Self {
            rank: Rank::from((n.trailing_zeros() / 4) as u8),
            suit: Suit::from((n.trailing_zeros() % 4) as u8),
        }
    }
}
impl Display for Card {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

use super::{rank::Rank, suit::Suit};
use std::fmt::{Display, Formatter, Result};
