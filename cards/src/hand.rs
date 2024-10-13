use super::{card::Card, suit::Suit};

/// Hand represents an unordered set of Cards. only in the limit, it is more memory efficient than Vec<Card>, ... but also, an advantage even for small N is that we avoid heap allocation. nice to use a single word for the full Hand independent of size stored as a u64, but only needs LSB bitstring of 52 bits. Each bit represents a unique card in the (unordered) set.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hand(u64);

impl Hand {
    pub fn empty() -> Self {
        Self(0)
    }
    #[cfg(feature = "std")]
    pub fn random() -> Self {
        let ref mut rng = rand::thread_rng();
        let cards = rand::Rng::gen::<u64>(rng);
        let cards = cards & Self::mask();
        Self(cards)
    }

    pub fn add(lhs: Self, rhs: Self) -> Self {
        assert!(u64::from(lhs) & u64::from(rhs) == 0);
        Self(lhs.0 | rhs.0)
    }

    pub fn size(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn complement(&self) -> Self {
        Self(self.0 ^ Self::mask())
    }

    pub fn draw(&mut self) -> Card {
        let card = Card::from(self.0);
        self.remove(card);
        card
    }
    pub fn remove(&mut self, card: Card) {
        let card = u8::from(card);
        let mask = !(1 << card);
        self.0 = self.0 & mask;
    }

    pub fn suit_count(&self) -> [u8; 4] {
        Suit::all()
            .map(|s| u64::from(s))
            .map(|u| (u & u64::from(self.0)))
            .map(|n| n.count_ones() as u8)
    }
    pub fn suited(&self, suit: &Suit) -> u16 {
        u16::from(Self::from(u64::from(*self) & u64::from(*suit)))
    }

    const fn mask() -> u64 {
        0x000FFFFFFFFFFFFF
    }
}

/// we can empty a hand from high to low
/// by removing the highest card until the hand is empty
impl Iterator for Hand {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        if self.size() == 0 {
            None
        } else {
            Some(self.draw())
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
#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
impl From<Vec<Card>> for Hand {
    fn from(cards: Vec<Card>) -> Self {
        Self(
            cards
                .into_iter()
                .map(|c| u64::from(c))
                .fold(0u64, |a, b| a | b),
        )
    }
}

/// one-way conversion to u16 Rank masks
/// zero-allocation, zero iteration. just shredding bits
impl From<Hand> for u16 {
    fn from(h: Hand) -> Self {
        let mut x = u64::from(h);
        let mut y = u16::default();
        x |= x >> 1;
        x |= x >> 2;
        x &= 0x1111111111111;
        y |= ((x >> 00) & 0001) as u16;
        y |= ((x >> 03) & 0002) as u16;
        y |= ((x >> 06) & 0004) as u16;
        y |= ((x >> 09) & 0008) as u16;
        y |= ((x >> 12) & 0016) as u16;
        y |= ((x >> 15) & 0032) as u16;
        y |= ((x >> 18) & 0064) as u16;
        y |= ((x >> 21) & 0128) as u16;
        y |= ((x >> 24) & 0256) as u16;
        y |= ((x >> 27) & 0512) as u16;
        y |= ((x >> 30) & 1024) as u16;
        y |= ((x >> 33) & 2048) as u16;
        y |= ((x >> 36) & 4096) as u16;
        y
    }
}

/// str isomorphism
/// this follows from Vec<Card> isomorphism
#[cfg(feature = "std")]
impl From<&str> for Hand {
    fn from(s: &str) -> Self {
        Self::from(
            s.split_whitespace()
                .map(|s| Card::from(s))
                .collect::<Vec<Card>>(),
        )
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for card in Vec::<Card>::from(*self) {
            write!(f, "{}", card)?;
        }
        Ok(())
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
    fn draw_iterator() {
        let mut iter = Hand::from("Jc Ts 2c Js").into_iter();
        assert_eq!(iter.next(), Some(Card::from("Js")));
        assert_eq!(iter.next(), Some(Card::from("Jc")));
        assert_eq!(iter.next(), Some(Card::from("Ts")));
        assert_eq!(iter.next(), Some(Card::from("2c")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn suit_masks() {
        let hand = Hand::from("2c 3d 4h 5s 6c 7d 8h 9s Tc Jd Qh Ks Ac");
        assert_eq!(hand.suited(&Suit::C), 0b_1000100010001); // C (2c, 6c, Tc, Ac)
        assert_eq!(hand.suited(&Suit::D), 0b_0001000100010); // D (3d, 7d, Jd)
        assert_eq!(hand.suited(&Suit::H), 0b_0010001000100); // H (4h, 8h, Qh)
        assert_eq!(hand.suited(&Suit::S), 0b_0100010001000); // S (5s, 9s, Ks)
    }
}
