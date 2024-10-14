use super::hand::Hand;

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
    mask: u64,
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
            // // ALTERNATE IMPL: mask at return, iterate as-is
            // (64 - 52) > self.next.leading_zeros() - self.mask.count_ones()
            // // CURRENT IMPL: mask at iteration, return as-is
            (64 - 52) > self.next.leading_zeros()
        }
    }

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

    fn current(&self) -> Hand {
        // // ALTERNATE IMPL: mask at return, iterate as-is
        // let mut returned_bits = 0;
        // let mut shifting_bits = self.next;
        // let mut excluded_bits = self.mask;
        // while excluded_bits > 0 {
        //     let lsbs = (1 << excluded_bits.trailing_zeros()) - 1;
        //     let msbs = !lsbs;
        //     returned_bits = returned_bits /* carry lsbs */ | (shifting_bits & lsbs);
        //     excluded_bits = excluded_bits /* erase mask */ & (excluded_bits - 1);
        //     shifting_bits = shifting_bits /* erase lsbs */ & msbs;
        //     shifting_bits = shifting_bits /* shift left */ << 1;
        // }
        // Hand::from(returned_bits | shifting_bits)
        // // CURRENT IMPL: mask at iteration, return as-is
        Hand::from(self.next)
    }

    fn advance(&mut self) {
        // // ALTERNATE IMPL: mask at return, iterate as-is
        // self.next = self.permute();
        // // CURRENT IMPL: mask at iteration, return as-is
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
            let hand = self.current();
            self.advance();
            Some(hand)
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
            mask: u64::from(mask),
        };
        while this.next & this.mask > 0 {
            this.next = this.permute();
        }
        this
    }
}

/// we can also "start" from a given Hand, and a mask.
impl From<(Hand, Hand)> for HandIterator {
    fn from((hand, mask): (Hand, Hand)) -> Self {
        let next = u64::from(hand);
        let mask = u64::from(mask);
        assert!(next & mask == 0);
        Self { next, mask }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_choose_three() {
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
    fn five_choose_three_with_mask() {
        let mask = Hand::from(0b______________________11_0);
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
    }
}
