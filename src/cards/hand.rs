use super::card::Card;

/// Hand is a bitstring of 52 bits
/// stored as a u64
/// each bit represents a card in the (unordered) set
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hand(u64);

impl Default for Hand {
    fn default() -> Self {
        Hand(1) // Set the default value to 2c
    }
}

/// u64 isomorphism
impl From<u64> for Hand {
    fn from(n: u64) -> Self {
        Self(n)
    }
}
impl From<Hand> for u64 {
    fn from(hand: Hand) -> Self {
        hand.0
    }
}

/// Vec<Card> isomorphism
/// we SUM/OR the cards to get the bitstring
impl From<Vec<Card>> for Hand {
    fn from(cards: Vec<Card>) -> Self {
        Self(cards.into_iter().map(|c| u64::from(c)).sum())
    }
}
/// we pluck the 1s out of the bitstring and convert them to cards
impl From<Hand> for Vec<Card> {
    fn from(hand: Hand) -> Self {
        let mut value = hand.0;
        let mut index = 0u8;
        let mut cards = Vec::new();
        while value != 0 {
            if value & 1 == 1 {
                cards.push(Card::from(index));
            }
            value = value >> 1;
            index = index + 1;
        }
        cards.reverse();
        cards
    }
}

/// HandIterator allows you to block certain cards and iterate over all possible hands of length n
/// n can be:
/// - inferred from length of initial cards
/// - specified directly by From<usize> for HandIterator
/// it is a struct that holds a u64 (and mask) and iterates over all possible hands under that mask
pub struct HandIterator {
    hand: Hand,
    last: Hand,
    mask: Hand,
}

impl HandIterator {
    fn blocks(&self, hand: Hand) -> bool {
        (self.mask.0 & hand.0) != 0
    }
    fn permute(&self) -> Hand {
        let x = self.hand.0;
        let  a = /* 000_100 || 000_011 -> 000_111 */ x | (x - 1);
        let  b = /*            000_111 -> 001_000 */ a + 1;
        let  c = /*            000_111 -> 111_000 */ !a;
        let  d = /* 111_000 && 001_000 -> 001_000 */ c & b;
        let  e = /*            001_000 -> 000_111 */ d - 1;
        let  f = /*            000_100 >>     xxx */ 1 + x.trailing_zeros();
        let  g = /*            000_111 -> 000_000 */ e >> f;
        let  h = /* 001_000 || 000_000 -> 001_000 */ b | g;
        Hand(h)
    }
}

/// iterator over Hand(u64)
impl Iterator for HandIterator {
    type Item = Hand;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.hand.0.leading_zeros() < 12 {
                return None;
            }
            self.last = self.hand;
            self.hand = self.permute();
            if self.blocks(self.hand) {
                continue;
            }
            return Some(self.last);
        }
    }
}

/// specifying the length of the hand with no mask
impl From<Hand> for HandIterator {
    fn from(hand: Hand) -> Self {
        Self {
            hand,
            last: Hand::default(),
            mask: Hand::default(),
        }
    }
}
