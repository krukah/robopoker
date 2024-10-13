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
    #[cfg(feature = "std")]
    pub fn draw() -> Card {
        use rand::Rng;
        let ref mut rng = rand::thread_rng();
        Card::from(rng.gen_range(0..52) as u8)
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

/// u64 isomorphism
/// each card is just one bit turned on
/// Ts
/// xxxxxxxxxxxx 0000000000001000000000000000000000000000000000000000
impl From<u64> for Card {
    fn from(n: u64) -> Self {
        Self::from(63 - n.leading_zeros() as u8)
    }
}
impl From<Card> for u64 {
    fn from(c: Card) -> u64 {
        1 << u8::from(c)
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.rank(), self.suit())
    }
}

/// str isomorphism
    #[cfg(feature = "std")]
impl From<&str> for Card {
    fn from(s: &str) -> Self {
        assert!(s.len() == 2);
        let rank = Rank::from(&s[0..1]);
        let suit = Suit::from(&s[1..2]);
        Card::from((rank, suit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_rank_suit() {
        let card = Card::draw();
        let suit = card.suit();
        let rank = card.rank();
        assert!(card == Card::from((rank, suit)));
    }

    #[test]
    fn bijective_u8() {
        let card = Card::draw();
        assert!(card == Card::from(u8::from(card)));
    }

    #[test]
    fn bijective_u32() {
        let card = Card::draw();
        assert!(card == Card::from(u32::from(card)));
    }

    #[test]
    fn bijective_u64() {
        let card = Card::draw();
        assert!(card == Card::from(u64::from(card)));
    }
}
