use super::rank::Rank;
use super::suit::Suit;

/// Card represents a playing card
/// it is a tuple of Rank and Suit
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card(u8);

impl Card {
    pub fn rank(&self) -> Rank {
        Rank::from(self.0 / 4)
    }
    pub fn suit(&self) -> Suit {
        Suit::from(self.0 % 4)
    }
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
        let suit = (1 << 13) << u8::from(c.suit());
        let rank = u16::from(c.rank()) as u32;
        rank | suit
    }
}
impl From<u32> for Card {
    fn from(n: u32) -> Self {
        let rank = Rank::from(n as u16);
        let suit = Suit::from((n >> 13).trailing_zeros() as u8);
        Self::from((rank, suit))
    }
}

/// u64 representation
/// each card is just one bit turned on. this is a one-way morphism
/// Ts
/// xxxxxxxxxxxx 0000000000001000000000000000000000000000000000000000
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

/// str isomorphism
impl TryFrom<&str> for Card {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.trim().len() {
            2 => {
                let rank = Rank::try_from(&s.trim()[0..1])?;
                let suit = Suit::try_from(&s.trim()[1..2])?;
                Ok(Card::from((rank, suit)))
            }
            _ => Err("2 characters".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::deck::Deck;

    #[test]
    fn bijective_rank_suit() {
        let card = Deck::new().draw();
        let suit = card.suit();
        let rank = card.rank();
        assert!(card == Card::from((rank, suit)));
    }

    #[test]
    fn bijective_u8() {
        let card = Deck::new().draw();
        assert!(card == Card::from(u8::from(card)));
    }

    #[test]
    fn bijective_u32() {
        let card = Deck::new().draw();
        assert!(card == Card::from(u32::from(card)));
    }
}
