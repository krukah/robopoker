use super::hand::Hand;

/// Combinatorial iterator over all n-card hands from a deck.
///
/// Generates all C(k, n) combinations of n cards from the k available cards
/// (those not blocked by the mask). Uses bit-twiddling to generate successive
/// permutations without storing them in memory.
///
/// # Construction
///
/// Created from `(n, mask)` where `n` is the hand size and `mask` is a [`Hand`]
/// of cards to exclude (already dealt cards).
///
/// # Performance
///
/// - Memory: O(1) — only stores current state, not all combinations
/// - Time per `.next()`: O(1) amortized via Gosper's hack
/// - Deterministic ordering for reproducible iteration
pub struct HandIterator {
    next: u64,
    mask: u64,
}

impl HandIterator {
    /// Total number of remaining combinations.
    ///
    /// Computes C(available, hand_size) using the multiplicative formula.
    pub fn combinations(&self) -> usize {
        let n = 52 - Hand::from(self.mask).size();
        let k = Hand::from(self.next).size();
        (0..k).fold(1, |x, i| x * (n - i) / (i + 1))
    }
    /// Tests whether iteration is complete.
    fn exhausted(&self) -> bool {
        if self.next == 0 {
            true
        } else {
            (64 - 52) > self.next.leading_zeros()
        }
    }
    /// Gosper's hack for next bit permutation with same popcount.
    ///
    /// See: https://graphics.stanford.edu/~seander/bithacks.html#NextBitPermutation
    fn permute(&self) -> u64 {
        let  x = /* 000_100                       */ self.next;
        let  a = /* 000_111 <- 000_100 || 000_110 */ x | (x - 1);
        let  b = /* 001_000 <-                    */ a + 1;
        let  c = /* 111_000 <-                    */ !   a;
        let  d = /* 001_000 <- 111_000 && 001_000 */ c & b;
        let  e = /* 000_111 <-                    */ d - 1;
        let  f = /*         << xxx                */ 1 + x.trailing_zeros();
        let  g = /* 000_000 <-                    */ e >> f;
        let  h = /* 001_000 <- 001_000 || 000_000 */ b | g;
        h
    }
    fn look(&self) -> Hand {
        Hand::from(self.next)
    }
    fn advance(&mut self) {
        loop {
            self.next = self.permute();
            if self.next & self.mask == 0 {
                break;
            }
        }
    }
}

impl Iterator for HandIterator {
    type Item = Hand;
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted() {
            None
        } else {
            let last = self.look();
            self.advance();
            Some(last)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let combos = self.combinations();
        (combos, Some(combos))
    }
}

/// size and mask are immutable and must be decided at construction
impl From<(usize, Hand)> for HandIterator {
    fn from((n, mask): (usize, Hand)) -> Self {
        let mut this = Self {
            next: (1 << n) - 1,
            #[cfg(feature = "shortdeck")]
            mask: u64::from(mask) | 0xFFFF, // remove 2-5 cards
            #[cfg(not(feature = "shortdeck"))]
            mask: u64::from(mask),
        };
        while this.next & this.mask > 0 && !this.exhausted() {
            this.next = this.permute();
        }
        this
    }
}

#[cfg(test)]
#[cfg(not(feature = "shortdeck"))]
mod tests {
    use super::*;

    #[test]
    fn n_choose_0() {
        let iter = HandIterator::from((0, Hand::empty()));
        assert_eq!(iter.count(), 0);
    }
    #[test]
    fn n_choose_1() {
        let iter = HandIterator::from((1, Hand::empty()));
        assert_eq!(iter.count(), Hand::from(Hand::mask()).size());
    }
    #[test]
    fn n_choose_2() {
        let iter = HandIterator::from((2, Hand::empty()));
        assert_eq!(iter.count(), 1326);
    }
    #[test]
    fn n_choose_0_mask_4() {
        let mask = Hand::from(0xF);
        let iter = HandIterator::from((0, mask));
        assert_eq!(iter.count(), 0);
    }
    #[test]
    fn n_choose_1_mask_4() {
        let mask = Hand::from(0xF);
        let iter = HandIterator::from((1, mask));
        assert_eq!(iter.count(), 48);
    }
    #[test]
    fn n_choose_2_mask_4() {
        let mask = Hand::from(0xF);
        let iter = HandIterator::from((2, mask));
        assert_eq!(iter.count(), 1128);
    }
    #[test]
    fn choose_3() {
        let mut iter = HandIterator::from((3, Hand::empty()));
        assert!(iter.next() == Some(Hand::from(0b00111)));
        assert!(iter.next() == Some(Hand::from(0b01011)));
        assert!(iter.next() == Some(Hand::from(0b01101)));
        assert!(iter.next() == Some(Hand::from(0b01110)));
        assert!(iter.next() == Some(Hand::from(0b10011)));
        assert!(iter.next() == Some(Hand::from(0b10101)));
        assert!(iter.next() == Some(Hand::from(0b10110)));
        assert!(iter.next() == Some(Hand::from(0b11001)));
        assert!(iter.next() == Some(Hand::from(0b11010)));
        assert!(iter.next() == Some(Hand::from(0b11100)));
    }
    #[test]
    fn choose_3_from_5() {
        let mask = Hand::from(0b_________________1111_00_1).complement();
        let mut iter = HandIterator::from((3, mask));
        assert!(iter.next() == Some(Hand::from(0b0011_00_1)));
        assert!(iter.next() == Some(Hand::from(0b0101_00_1)));
        assert!(iter.next() == Some(Hand::from(0b0110_00_1)));
        assert!(iter.next() == Some(Hand::from(0b0111_00_0)));
        assert!(iter.next() == Some(Hand::from(0b1001_00_1)));
        assert!(iter.next() == Some(Hand::from(0b1010_00_1)));
        assert!(iter.next() == Some(Hand::from(0b1011_00_0)));
        assert!(iter.next() == Some(Hand::from(0b1100_00_1)));
        assert!(iter.next() == Some(Hand::from(0b1101_00_0)));
        assert!(iter.next() == Some(Hand::from(0b1110_00_0)));
        assert!(iter.next() == None);
    }
}

#[cfg(test)]
#[cfg(feature = "shortdeck")]
mod tests_shortdeck {
    use super::*;

    #[test]
    fn choose_2_shortdeck() {
        let mut iter = HandIterator::from((2, Hand::from(0)));
        assert_eq!(iter.next(), Some(Hand::from(0b110000000000000000)));
    }
}
