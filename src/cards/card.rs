use super::{hand::Hand, rank::Rank, suit::Suit};

/// Card represents a playing card
/// it is a tuple of Rank and Suit
/// actually we may as well want to store this as a u8
/// we expose Rank and Suit via pub methods anyway
/// and there's not enough entropy to need 16 bits
/// low-hanging optimization to cut Card memory in half

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card(u8);
impl Card {
    pub fn rank(&self) -> Rank {
        Rank::from(self.0 / 4)
    }
    pub fn suit(&self) -> Suit {
        Suit::from(self.0 % 4)
    }
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(51);
}

/// (Rank, Suit) isomorphism
impl From<(Rank, Suit)> for Card {
    fn from((r, s): (Rank, Suit)) -> Self {
        Self(u8::from(r) * 4 + u8::from(s))
    }
}

/// u8 isomorphism
/// each card is mapped to its location in a sorted deck 1-52
/// Ts
/// 39
/// 0b00100111
impl From<Card> for u8 {
    fn from(c: Card) -> u8 {
        c.0
    }
}
impl From<u8> for Card {
    fn from(n: u8) -> Self {
        Self(n)
    }
}

/// u32 isomorphism
/// a Card is bitwise OR. Suit and Rank are bitmasks of the 17 LSBs (so close to u16, alas)
/// Ts
/// xxxxxxxxxxxxxxx cdhs AKQJT98765432
/// 000000000000000 0010 0000100000000
impl From<Card> for u32 {
    fn from(c: Card) -> u32 {
        u32::from(c.suit()) | u32::from(c.rank())
    }
}
impl From<u32> for Card {
    fn from(n: u32) -> Self {
        Self::from((Rank::from(n), Suit::from(n)))
    }
}

/// u64 isomorphism
/// each card is just one bit turned on
/// Ts
/// xxxxxxxxxxxx 0000000000001000000000000000000000000000000000000000
impl From<u64> for Card {
    fn from(n: u64) -> Self {
        Self(n.trailing_zeros() as u8)
    }
}
impl From<Card> for u64 {
    fn from(c: Card) -> u64 {
        1 << u8::from(c)
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.rank(), self.suit())
    }
}

/// CardIterator is an iterator over all single cards in a deck
/// it is memory-efficient, using a mask to skip over cards
/// it is deterministic, as it will always iterate over the same cards in the same order
/// it is lazy, as it generates cards on the fly (no heap allocation)
/// it is fast, as it uses bitwise operations

pub struct CardIterator {
    card: Card,
    last: Card,
    mask: Hand,
}
impl CardIterator {
    fn exhausted(&self) -> bool {
        self.last == Card::MAX
    }
    fn blocks(&self, card: Card) -> bool {
        (u64::from(self.mask) & u64::from(card)) != 0
    }
    fn turn(&self) -> Card {
        Card::from((u8::from(self.card) + 1) % 52)
    }
}
impl Iterator for CardIterator {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.exhausted() {
                return None;
            }
            self.last = self.card;
            self.card = self.turn();
            if self.blocks(self.card) {
                continue;
            } else {
                return Some(self.last);
            }
        }
    }
}
