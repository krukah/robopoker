use crate::cards::hand::Hand;
use crate::cards::hole::Hole;
use crate::transport::support::Support;
use crate::Probability;
use std::hash::Hash;
use std::u64;

/// Abstraction represents a lookup value for a given set of Observations.
///
/// - River: we use a u8 to represent the equity bucket, i.e. Equity(0) is the worst bucket, and Equity(50) is the best bucket.
/// - Pre-Flop: we do not use any abstraction, rather store the 169 strategically-unique hands as u64.
/// - Other Streets: we use a u64 to represent the hash signature of the centroid Histogram over lower layers of abstraction.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub enum Abstraction {
    Percent(u8),   // river
    Learned(u64),  // flop, turn
    Preflop(Hole), // preflop
}

impl Support for Abstraction {}

impl Abstraction {
    pub fn random() -> Self {
        Self::Learned(loop {
            let x = rand::random::<u64>();
            match x >> 52 {
                POCKET_TAG => continue,
                EQUITY_TAG => continue,
                _ => break x,
            }
        })
    }

    fn quantize(p: Probability) -> u8 {
        (p * Probability::from(Self::N)).round() as u8
    }
    fn floatize(q: u8) -> Probability {
        Probability::from(q) / Probability::from(Self::N)
    }

    const N: u8 = 63;
    const BUCKETS: [Self; Self::N as usize + 1] = Self::buckets();
    const fn buckets() -> [Self; Self::N as usize + 1] {
        let mut buckets = [Self::Percent(0); Self::N as usize + 1];
        let mut i = 0;
        while i <= Self::N {
            buckets[i as usize] = Self::Percent(i as u8);
            i += 1;
        }
        buckets
    }
    pub const fn range() -> &'static [Self] {
        &Self::BUCKETS
    }
    pub const fn size() -> usize {
        Self::N as usize
    }
}

/// probability isomorphism
///
/// for river, we use a u8 to represent the equity bucket,
/// i.e. Equity(0) is the 0% equity bucket,
/// and Equity(N) is the 100% equity bucket.
impl From<Probability> for Abstraction {
    fn from(p: Probability) -> Self {
        assert!(p >= 0.);
        assert!(p <= 1.);
        Self::Percent(Abstraction::quantize(p))
    }
}
impl From<Abstraction> for Probability {
    fn from(abstraction: Abstraction) -> Self {
        match abstraction {
            Abstraction::Percent(n) => Abstraction::floatize(n),
            Abstraction::Learned(_) => unreachable!("no cluster into probability"),
            Abstraction::Preflop(_) => unreachable!("no preflop into probability"),
        }
    }
}

const EQUITY_TAG: u64 = 0xEEE;
const POCKET_TAG: u64 = 0xFFF;
/// u64 isomorphism
///
/// conversion to u64 for SQL storage.
impl From<Abstraction> for u64 {
    fn from(a: Abstraction) -> Self {
        match a {
            Abstraction::Learned(n) => n,
            Abstraction::Percent(e) => (EQUITY_TAG << 52) | (e as u64 & 0xFF) << 44,
            Abstraction::Preflop(h) => (POCKET_TAG << 52) | u64::from(Hand::from(h)),
        }
    }
}
impl From<u64> for Abstraction {
    fn from(n: u64) -> Self {
        match n >> 52 {
            EQUITY_TAG => Self::Percent(((n >> 44) & 0xFF) as u8),
            POCKET_TAG => Self::Preflop(Hole::from(Hand::from(n & 0x000FFFFFFFFFFFFF))),
            _ => Self::Learned(n),
        }
    }
}

/// i64 isomorphism
///
/// conversion to i64 for SQL storage.
impl From<Abstraction> for i64 {
    fn from(abstraction: Abstraction) -> Self {
        u64::from(abstraction) as i64
    }
}
impl From<i64> for Abstraction {
    fn from(n: i64) -> Self {
        Self::Learned(n as u64)
    }
}

/// lossless preflop abstraction
impl From<Hole> for Abstraction {
    fn from(hole: Hole) -> Self {
        Self::Preflop(hole)
    }
}

impl crate::Arbitrary for Abstraction {
    fn random() -> Self {
        Self::random()
    }
}

impl std::fmt::Display for Abstraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Learned(n) => write!(f, "{:016x}", n),
            Self::Percent(n) => write!(f, "Equity({:00.2})", Self::floatize(*n)),
            Self::Preflop(h) => write!(f, "Pocket({})", h),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;

    #[test]
    fn is_quantize_inverse_floatize() {
        for p in (0..=100).map(|x| x as Probability / 100.) {
            let q = Abstraction::quantize(p);
            let f = Abstraction::floatize(q);
            assert!((p - f).abs() < 1. / Abstraction::N as Probability);
        }
    }

    #[test]
    fn is_floatize_inverse_quantize() {
        for q in 0..=Abstraction::N {
            let p = Abstraction::floatize(q);
            let i = Abstraction::quantize(p);
            assert!(q == i);
        }
    }

    #[test]
    fn bijective_u64_random() {
        let random = Abstraction::random();
        assert_eq!(random, Abstraction::from(u64::from(random)));
    }

    #[test]
    fn bijective_u64_equity() {
        let equity = Abstraction::Percent(Abstraction::N / 2);
        assert_eq!(equity, Abstraction::from(u64::from(equity)));
    }

    #[test]
    fn bijective_u64_pocket() {
        let pocket = Abstraction::Preflop(Hole::from(Observation::from(Street::Pref)));
        assert_eq!(pocket, Abstraction::from(u64::from(pocket)));
    }
}
