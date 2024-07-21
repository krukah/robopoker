use super::card::Card;

/// Hand represents an unordered set of Cards.
/// only in the limit, it is more memory efficient than Vec<Card>, ...
/// but also, an advantage even for small N is that we avoid heap allocation.
/// nice to use a single word for the full Hand independent of size
/// stored as a u64, but only needs LSB bitstring of 52 bits
/// each bit represents a unique card in the (unordered) set
/// if necessary, we can modify logic to account for strategy-isomorphic Hands !!
/// i.e. break a symmetry across suits when no flushes are present
/// although this might only be possible at the Observation level
/// perhaps Hand has insufficient information

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hand(u64);
impl Hand {
    pub fn size(&self) -> u8 {
        self.0.count_ones() as u8
    }
}

/// Group operation on Hand follow bitwise OR
impl std::ops::Add for Hand {
    type Output = Hand;
    fn add(self, rhs: Hand) -> Hand {
        Hand(self.0 | rhs.0)
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
        while value != 0 {
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
        Self(cards.iter().map(|c| u64::from(*c)).fold(0, |a, b| a | b))
    }
}

impl std::fmt::Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", Vec::<Card>::from(*self))
    }
}

/// HandIterator allows you to block certain cards and iterate over all possible hands of length n
/// n can be:
/// - inferred from length of initial cards
/// - specified directly by From<usize> for HandIterator
/// it is a struct that holds a u64 (and mask) and iterates over all possible hands under that mask
/// it is memory efficient because it does not store all possible hands
/// it is deterministic because it always iterates in the same order
/// it is fast because it uses bitwise operations

pub struct HandIterator {
    hand: Hand,
    last: Hand,
    mask: Hand,
}
impl HandIterator {
    fn exhausted(&self) -> bool {
        self.hand.0.leading_zeros() < 12 || self.hand.0 == 0
    }
    fn blocks(&self, hand: Hand) -> bool {
        (self.mask.0 & hand.0) != 0
    }
    fn permute(&self) -> Hand {
        let  x = /* 000_100                       */ self.hand.0;
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
impl Iterator for HandIterator {
    type Item = Hand;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.exhausted() {
                return None;
            }
            self.last = self.hand;
            self.hand = self.permute();
            if self.blocks(self.hand) {
                continue;
            } else {
                return Some(self.last);
            }
        }
    }
}

/// size and mask are immutable and must be decided at construction
impl From<(usize, Hand)> for HandIterator {
    fn from((size, mask): (usize, Hand)) -> Self {
        Self {
            hand: Hand((1 << size) - 1),
            last: Hand(0),
            mask,
        }
    }
}
