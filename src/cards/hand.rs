use super::card::Card;

/// Hand represents an unordered set of Cards. only in the limit, it is more memory efficient than Vec<Card>, ... but also, an advantage even for small N is that we avoid heap allocation. nice to use a single word for the full Hand independent of size stored as a u64, but only needs LSB bitstring of 52 bits. Each bit represents a unique card in the (unordered) set.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hand(u64);

impl Hand {
    pub fn empty() -> Self {
        Self(0)
    }
    pub fn size(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn add(lhs: Self, rhs: Self) -> Self {
        assert!(u64::from(lhs) & u64::from(rhs) == 0);
        Self(lhs.0 | rhs.0)
    }
    pub fn complement(&self) -> Self {
        Self(self.0 ^ Self::mask())
    }
    pub fn random() -> Self {
        let ref mut rng = rand::thread_rng();
        let cards = rand::Rng::gen::<u64>(rng);
        let cards = cards & Self::mask();
        Self(cards)
    }
    pub fn suit_count(&self) -> [u8; 4] {
        crate::cards::suit::Suit::all()
            .map(|s| u64::from(s))
            .map(|u| (u & u64::from(self.0)))
            .map(|n| n.count_ones() as u8)
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
        Self(
            cards
                .into_iter()
                .map(|c| u64::from(c))
                .fold(0u64, |a, b| a | b),
        )
    }
}

/// str isomorphism
/// this follows from Vec<Card> isomorphism
impl From<&str> for Hand {
    fn from(s: &str) -> Self {
        Self::from(
            s.split_whitespace()
                .map(|s| Card::from(s))
                .collect::<Vec<Card>>(),
        )
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
        let mut iter = Hand::from("2c Ts Jc Js").into_iter();
        assert_eq!(iter.next(), Some(Card::from("Js")));
        assert_eq!(iter.next(), Some(Card::from("Jc")));
        assert_eq!(iter.next(), Some(Card::from("Ts")));
        assert_eq!(iter.next(), Some(Card::from("2c")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn draw_singular() {
        let mut hand = Hand::from("2c");
        assert_eq!(hand.draw(), Card::from("2c"));
        assert_eq!(hand.size(), 0);
    }
}
