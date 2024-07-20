use super::{hand::Hand, rank::Rank, suit::Suit};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

    pub const MAX: Self = Self {
        rank: Rank::MAX,
        suit: Suit::MAX,
    };
    pub const MIN: Self = Self {
        rank: Rank::MIN,
        suit: Suit::MIN,
    };
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
impl From<u64> for Card {
    fn from(n: u64) -> Self {
        Self {
            rank: Rank::from((n.trailing_zeros() / 4) as u8),
            suit: Suit::from((n.trailing_zeros() % 4) as u8),
        }
    }
}
impl From<Card> for u64 {
    fn from(c: Card) -> u64 {
        1 << u8::from(c)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

/// A memory-efficient deterministic Card Iterator.
#[derive(Default)]
pub struct CardIterator {
    card: Card,
    last: Card,
    mask: Hand,
}

/// we interface with the Iterator by adding and removing cards from the mask, or by seeking to a specific card.
/// internally, for impl Iterator::Card, we use ::reveals() to give us the next valid card, which uses ::ignores() to inform which to skip.
impl CardIterator {
    fn blocks(&self, card: Card) -> bool {
        u64::from(self.mask) & u64::from(card) != 0
    }
    fn reveal(&self) -> Card {
        Card::from((u8::from(self.card) + 1) % 52)
    }
}

/// we skip over masked cards, effectively are removed from the deck
impl Iterator for CardIterator {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.last == Card::MAX {
                return None;
            }
            self.last = self.card;
            self.card = self.reveal();
            if self.blocks(self.card) {
                continue;
            }
            return Some(self.last);
        }
    }
}

/// we can construct an iterator to start after a specific card without a mask
impl From<Card> for CardIterator {
    fn from(card: Card) -> Self {
        Self {
            card,
            last: Card::default(),
            mask: Hand::default(),
        }
    }
}

/// we can also start after Card::MIN and start with a specific mask
impl From<Hand> for CardIterator {
    fn from(mask: Hand) -> Self {
        Self {
            card: Card::default(),
            last: Card::default(),
            mask,
        }
    }
}
