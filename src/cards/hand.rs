use super::card::Card;

/// Hand represents an unordered set of Cards. only in the limit, it is more memory efficient than Vec<Card>, ... but also, an advantage even for small N is that we avoid heap allocation. nice to use a single word for the full Hand independent of size stored as a u64, but only needs LSB bitstring of 52 bits each bit represents a unique card in the (unordered) set if necessary, we can modify logic to account for strategy-isomorphic Hands !! i.e. break a symmetry across suits when no flushes are present although this might only be possible at the Observation level perhaps Hand has insufficient information
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hand(u64);
impl Hand {
    pub fn size(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn add(lhs: Self, rhs: Self) -> Self {
        Self(lhs.0 | rhs.0)
    }
    pub fn draw(&mut self) -> Card {
        let index = self.0.trailing_zeros();
        let card = Card::from(index as u8);
        self.0 &= !(1 << index);
        card
    }
    pub fn take(&mut self, card: Card) {
        self.0 |= 1 << u64::from(card);
    }
}

/// u64 isomorphism
/// we SUM/OR the cards to get the bitstring
/// [2c, Ts, Jc, Js]
/// xxxxxxxxxxxx 0000000010011000000000000000000000000000000000000001
impl From<u64> for Hand {
    fn from(n: u64) -> Self {
        Self(n)
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
        Self(cards.iter().map(|c| u64::from(*c)).fold(0u64, |a, b| a | b))
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
