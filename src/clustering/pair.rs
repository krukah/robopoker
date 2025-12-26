use crate::cards::*;
use crate::gameplay::*;

/// A unique identifier for a pair of abstractions.
/// Packed as: [2 bits street][30 bits triangular index]
/// where triangular index = j*(j-1)/2 + i for indices i < j
#[derive(Default, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct Pair(u32);

const STREET_SHIFT: u32 = 30;
const STREET_MASK: u32 = 0b11 << STREET_SHIFT;
const INDEX_MASK: u32 = (1 << STREET_SHIFT) - 1;

impl Pair {
    /// Construct a pair from street and two abstraction indices.
    /// Indices are automatically sorted so i < j.
    /// When i == j, returns a degenerate pair with triangular index 0.
    pub const fn new(street: Street, i: usize, j: usize) -> Self {
        let lo_bits = Self::merge(i, j) as u32;
        let hi_bits = street as u32;
        Self(hi_bits << STREET_SHIFT | lo_bits)
    }
    /// Extract the street from this pair.
    pub const fn street(&self) -> Street {
        match (self.0 & STREET_MASK) >> STREET_SHIFT {
            0 => Street::Pref,
            1 => Street::Flop,
            2 => Street::Turn,
            3 => Street::Rive,
            _ => unreachable!(),
        }
    }
    /// Extract the triangular index (for array indexing within a street's Metric).
    pub const fn triangular(&self) -> usize {
        (self.0 & INDEX_MASK) as usize
    }
    pub const fn split(t: usize) -> (usize, usize) {
        let j = ((1 + (1 + 8 * t).isqrt()) / 2) as usize;
        let i = t - j * (j - 1) / 2;
        (i, j)
    }
    pub const fn merge(i: usize, j: usize) -> usize {
        let (lo, hi) = if i < j { (i, j) } else { (j, i) };
        if hi == 0 { 0 } else { hi * (hi - 1) / 2 + lo }
    }
}

impl Pair {
    /// Recover the original (lo, hi) index pair from the triangular index.
    pub fn indices(&self) -> (usize, usize) {
        Self::split(self.triangular())
    }
    /// Recover the two abstractions that form this pair.
    pub fn abstractions(&self) -> (Abstraction, Abstraction) {
        let (i, j) = self.indices();
        let street = self.street();
        (
            Abstraction::from((street, i)),
            Abstraction::from((street, j)),
        )
    }
}

impl From<(&Abstraction, &Abstraction)> for Pair {
    fn from((a, b): (&Abstraction, &Abstraction)) -> Self {
        debug_assert!(a.street() == b.street());
        Self::new(a.street(), a.index(), b.index())
    }
}

impl From<Pair> for i32 {
    fn from(pair: Pair) -> Self {
        pair.0 as i32
    }
}

impl From<i32> for Pair {
    fn from(i: i32) -> Self {
        Self(i as u32)
    }
}

/// i64 isomorphism (convenience, for legacy compatibility)
#[doc(hidden)]
#[warn(deprecated)]
impl From<Pair> for i64 {
    fn from(pair: Pair) -> Self {
        i32::from(pair) as i64
    }
}

#[doc(hidden)]
#[warn(deprecated)]
impl From<i64> for Pair {
    fn from(i: i64) -> Self {
        Self::from(i as i32)
    }
}

impl From<Street> for Pair {
    fn from(street: Street) -> Self {
        let k = street.n_abstractions();
        let i = rand::random_range(0..k);
        let j = rand::random_range(0..k);
        Self::new(street, i, j)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bijective_pair_abstractions() {
        for street in Street::all() {
            let k = street.n_abstractions();
            for i in 0..k {
                for j in (i + 1)..k {
                    let a = Abstraction::from((street, i));
                    let b = Abstraction::from((street, j));
                    let pair = Pair::from((&a, &b));
                    let (lo, hi) = pair.abstractions();
                    assert_eq!(lo, a);
                    assert_eq!(hi, b);
                }
            }
        }
    }
    #[test]
    fn bijective_abstractions_pair() {
        for street in Street::all() {
            let k = street.n_abstractions();
            for i in 0..k {
                for j in (i + 1)..k {
                    let a = Abstraction::from((street, i));
                    let b = Abstraction::from((street, j));
                    let pair = Pair::from((&a, &b));
                    let (lo, hi) = pair.abstractions();
                    let roundtrip = Pair::from((&lo, &hi));
                    assert_eq!(pair, roundtrip);
                }
            }
        }
    }
    #[test]
    fn bijective_pair_abstractions_symmetry() {
        for street in Street::all() {
            let k = street.n_abstractions();
            for i in 0..k {
                for j in (i + 1)..k {
                    let a = Abstraction::from((street, i));
                    let b = Abstraction::from((street, j));
                    let pair_ab = Pair::from((&a, &b));
                    let pair_ba = Pair::from((&b, &a));
                    assert_eq!(pair_ab, pair_ba);
                }
            }
        }
    }
    #[test]
    fn pair_street_preserved() {
        for street in Street::all() {
            let k = street.n_abstractions();
            for i in 0..k {
                for j in (i + 1)..k {
                    let pair = Pair::new(street, i, j);
                    assert_eq!(pair.street(), street);
                    let (lo, hi) = pair.abstractions();
                    assert_eq!(lo.street(), street);
                    assert_eq!(hi.street(), street);
                }
            }
        }
    }
    #[test]
    fn split_merge_roundtrip() {
        for j in 1..256 {
            for i in 0..j {
                let t = Pair::merge(i, j);
                let (ii, jj) = Pair::split(t);
                assert_eq!((i, j), (ii, jj));
            }
        }
    }
    #[test]
    fn merge_split_roundtrip() {
        for t in 0..32768 {
            let (i, j) = Pair::split(t);
            let tt = Pair::merge(i, j);
            assert_eq!(t, tt);
        }
    }
}
