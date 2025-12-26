use super::card::Card;
use super::rank::Rank;
use super::suit::Suit;
use crate::Arbitrary;

/// Hand represents an unordered set of Cards. only in the limit, it is more memory efficient than Vec<Card>, ... but also, an advantage even for small N is that we avoid heap allocation. nice to use a single word for the full Hand independent of size stored as a u64, but only needs LSB bitstring of 52 bits. Each bit represents a unique card in the (unordered) set.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hand(u64);

impl Hand {
    pub fn empty() -> Self {
        Self(0)
    }

    pub fn or(lhs: Self, rhs: Self) -> Self {
        lhs + rhs
    }

    pub fn add(lhs: Self, rhs: Self) -> Self {
        assert!((lhs.0 & rhs.0) == 0);
        lhs + rhs
    }

    pub fn complement(&self) -> Self {
        Self(self.0 ^ Self::mask())
    }
    pub fn size(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn of(&self, suit: &Suit) -> Hand {
        let this = u64::from(*self);
        let mask = u64::from(*suit);
        Self::from(this & mask)
    }
    pub fn min_rank(&self) -> Option<Rank> {
        match self.size() {
            0 => None,
            _ => Some(Rank::lo(self.0)),
        }
    }
    pub fn max_rank(&self) -> Option<Rank> {
        match self.size() {
            0 => None,
            _ => Some(Rank::hi(self.0)),
        }
    }
    pub fn remove(&mut self, card: Card) {
        let card = u8::from(card);
        let mask = !(1 << card);
        self.0 &= mask;
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.0 & (1 << u8::from(*card)) != 0
    }

    pub fn shuffle(&self) -> Vec<Card> {
        use rand::seq::SliceRandom;
        let ref mut rng = rand::rng();
        let mut cards = Vec::<Card>::from(self.clone());
        cards.shuffle(rng);
        cards
    }

    /// one-way conversion to u16 Rank masks
    /// zero-allocation, zero iteration. just shredding bits
    pub fn ranks(&self) -> u16 {
        let mut x = self.0;
        x |= x >> 1;
        x |= x >> 2;
        x &= 0x1111111111111;
        let mut y = 0u64;
        y |= (x >> 00) & 0b_0000000000001;
        y |= (x >> 03) & 0b_0000000000010;
        y |= (x >> 06) & 0b_0000000000100;
        y |= (x >> 09) & 0b_0000000001000;
        y |= (x >> 12) & 0b_0000000010000;
        y |= (x >> 15) & 0b_0000000100000;
        y |= (x >> 18) & 0b_0000001000000;
        y |= (x >> 21) & 0b_0000010000000;
        y |= (x >> 24) & 0b_0000100000000;
        y |= (x >> 27) & 0b_0001000000000;
        y |= (x >> 30) & 0b_0010000000000;
        y |= (x >> 33) & 0b_0100000000000;
        y |= (x >> 36) & 0b_1000000000000;
        y as u16
    }

    #[cfg(not(feature = "shortdeck"))]
    pub const fn mask() -> u64 {
        0x000FFFFFFFFFFFFF
    }
    #[cfg(feature = "shortdeck")]
    pub const fn mask() -> u64 {
        0x000FFFFFFFFF0000
    }
}

/// we can empty a hand from high to low
/// by removing the highest card until the hand is empty
impl Iterator for Hand {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        match self.size() {
            0 => None,
            _ => {
                let card = self.0.trailing_zeros() as u8;
                let card = Card::from(card);
                self.remove(card);
                Some(card)
            }
        }
    }
}

/// u64 isomorphism
/// we SUM/OR the cards to get the bitstring
/// [2c, Ts, Jc, Js]
/// xxxxxxxxxxxx 0000000010011000000000000000000000000000000000000001
impl From<u64> for Hand {
    fn from(n: u64) -> Self {
        Self(n & Self::mask())
    }
}
impl From<Hand> for u64 {
    fn from(h: Hand) -> Self {
        h.0
    }
}

/// Vec<Card> isomorphism (up to Vec permutation, this always comes out sorted)
/// we SUM/OR the cards to get the bitstring
/// [2c, Ts, Jc, Js]
/// xxxxxxxxxxxx 0000000010011000000000000000000000000000000000000001
impl From<Hand> for Vec<Card> {
    fn from(h: Hand) -> Self {
        let mut value = h.0;
        let mut index = 0u8;
        let mut cards = Vec::new();
        while value > 0 {
            if value & 1 == 1 {
                cards.push(Card::from(index));
            }
            value = value >> 1;
            index = index + 1;
        }
        cards
    }
}
impl From<Vec<Card>> for Hand {
    fn from(cards: Vec<Card>) -> Self {
        cards.into_iter().collect()
    }
}

impl FromIterator<Card> for Hand {
    fn from_iter<I: IntoIterator<Item = Card>>(iter: I) -> Self {
        Self(iter.into_iter().map(u64::from).fold(0u64, |a, b| a | b))
    }
}

impl From<Hand> for u16 {
    fn from(h: Hand) -> Self {
        h.ranks()
    }
}

/// one-way conversion from Card
impl From<Card> for Hand {
    fn from(card: Card) -> Self {
        Self(1u64 << u8::from(card))
    }
}

/// str isomorphism
/// this follows from Vec<Card> isomorphism
impl TryFrom<&str> for Hand {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(Card::parse(s)?))
    }
}

impl std::ops::Add<Self> for Hand {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::fmt::Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for card in Vec::<Card>::from(*self) {
            write!(f, "{}", card)?;
        }
        Ok(())
    }
}

impl Arbitrary for Hand {
    fn random() -> Self {
        let cards = rand::random::<u64>();
        let cards = cards & Self::mask();
        Self(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_u64() {
        let hand = Hand::random();
        assert_eq!(hand, Hand::from(u64::from(hand)));
    }

    #[test]
    fn card_iteration() {
        let mut iter = Hand::try_from("Jc Ts 2c Js").unwrap().into_iter();
        assert_eq!(iter.next(), Some(Card::try_from("2c").unwrap()));
        assert_eq!(iter.next(), Some(Card::try_from("Ts").unwrap()));
        assert_eq!(iter.next(), Some(Card::try_from("Jc").unwrap()));
        assert_eq!(iter.next(), Some(Card::try_from("Js").unwrap()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[cfg(not(feature = "shortdeck"))]
    fn ranks_in_suit() {
        let hand = Hand::try_from("2c 3d 4h 5s 6c 7d 8h 9s Tc Jd Qh Ks Ac").unwrap();
        assert_eq!(u16::from(hand.of(&Suit::C)), 0b000_1000100010001); // C (2c, 6c, Tc, Ac)
        assert_eq!(u16::from(hand.of(&Suit::D)), 0b000_0001000100010); // D (3d, 7d, Jd)
        assert_eq!(u16::from(hand.of(&Suit::H)), 0b000_0010001000100); // H (4h, 8h, Qh)
        assert_eq!(u16::from(hand.of(&Suit::S)), 0b000_0100010001000); // S (5s, 9s, Ks)
    }
}
