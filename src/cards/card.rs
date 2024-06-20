#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    pub fn rank(&self) -> Rank {
        self.rank
    }
    pub fn suit(&self) -> Suit {
        self.suit
    }
}

/// u8 isomorphism
/// each card is mapped to its location in a sorted deck 1-52
/// Ts
/// 39
/// 0b00100111
impl From<Card> for u8 {
    fn from(c: Card) -> u8 {
        u8::from(c.suit) + u8::from(c.rank) * 4
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

/// u32 isomorphism
/// a Card is bitwise OR. Suit and Rank are bitmasks of the 17 LSBs
/// Ts
/// xxxxxxxxxxxxxxx cdhs AKQJT98765432
/// 000000000000000 0010 0000100000000
impl From<Card> for u32 {
    fn from(c: Card) -> u32 {
        u32::from(c.suit) | u32::from(c.rank)
    }
}
impl From<u32> for Card {
    fn from(n: u32) -> Self {
        Self {
            rank: Rank::from(n),
            suit: Suit::from(n),
        }
    }
}

/// u64 isomorphism
/// each card is just one bit turned on
/// Ts
/// xxxxxxxxxxxx 0000000000001000000000000000000000000000000000000000
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
