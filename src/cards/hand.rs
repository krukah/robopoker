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

/// TODO:
/// unclear if we should
/// skip over (52 choose m) masked bits in O(1)
/// or iterate over ((52 - m) choose n) bits in O(exp)
///
/// HandIterator allows you to block certain cards and iterate over all possible hands of length n
/// n can be:
/// - inferred from length of initial cards
/// - specified directly by From<usize> for HandIterator
/// it is a struct that holds a u64 (and mask) and iterates over all possible hands under that mask
/// it is memory efficient because it does not store all possible hands
/// it is deterministic because it always iterates in the same order
/// it is fast because it uses bitwise operations
/// it is flexible because it can be used to iterate over any subset of cards
pub struct HandIterator {
    next: u64,
    curr: u64,
    mask: u64,
}

/// size and mask are immutable and must be decided at construction
impl From<(usize, Hand)> for HandIterator {
    fn from((n_cards, mask): (usize, Hand)) -> Self {
        Self {
            next: (1 << n_cards) - 1,
            curr: 0u64,
            mask: u64::from(mask),
        }
    }
}

impl HandIterator {
    pub fn combinations(&self) -> usize {
        let n = 52 - Hand::from(self.mask).size(); // count_ones()
        let k = Hand::from(self.next).size(); // count_ones()
        (0..k).fold(1, |x, i| x * (n - i) / (i + 1))
    }

    fn exhausted(&self) -> bool {
        if self.next == 0 {
            true
        } else {
            self.next.leading_zeros() - self.mask.count_ones() < (64 - 52)
        }
    }

    fn jump(&self) -> Hand {
        // apply masking by shuffling around bits
        let mut returned_bits = 0;
        let mut shifting_bits = self.curr;
        let mut excluded_bits = self.mask;
        while excluded_bits > 0 {
            let lsbs = (1 << excluded_bits.trailing_zeros()) - 1;
            let msbs = !lsbs;
            returned_bits = returned_bits /* carry lsbs */ | (shifting_bits & lsbs);
            excluded_bits = excluded_bits /* erase mask */ & (excluded_bits - 1);
            shifting_bits = shifting_bits /* erase lsbs */ & msbs;
            shifting_bits = shifting_bits /* shift left */ << 1;
        }
        Hand(returned_bits | shifting_bits)
    }

    fn permute(&self) -> u64 {
        let  x = /* 000_100                       */ self.next; // == self.curr at this point
        let  a = /* 000_111 <- 000_100 || 000_110 */ x | (x - 1);
        let  b = /* 001_000 <-                    */ a + 1;
        let  c = /* 111_000 <-                    */ !a;
        let  d = /* 001_000 <- 111_000 && 001_000 */ c & b;
        let  e = /* 000_111 <-                    */ d - 1;
        let  f = /*         << xxx                */ 1 + x.trailing_zeros();
        let  g = /* 000_000 <-                    */ e >> f;
        let  h = /* 001_000 <- 001_000 || 000_000 */ b | g;
        h
    }
}

impl Iterator for HandIterator {
    type Item = Hand;
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted() {
            None
        } else {
            self.curr = self.next;
            self.next = self.permute();
            Some(self.jump())
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let combos = self.combinations();
        (combos, Some(combos))
    }
}
