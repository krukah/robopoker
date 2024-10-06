use crate::Probability;
use std::hash::Hash;
use std::u64;

/// Abstraction represents a lookup value for a given set of Observations.
///
/// - River: we use a i8 to represent the equity bucket, i.e. Equity(0) is the worst bucket, and Equity(50) is the best bucket.
/// - Pre-Flop: we do not use any abstraction, rather store the 169 strategically-unique hands as u64.
/// - Other Streets: we use a u64 to represent the hash signature of the centroid Histogram over lower layers of abstraction.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub enum Abstraction {
    Random(u64),
    Equity(i8),
}

impl Abstraction {
    const N: i8 = 50;
    pub fn random() -> Self {
        Self::Random(rand::random::<u64>())
    }

    fn quantize(p: Probability) -> i8 {
        (p * Probability::from(Self::N)).round() as i8
    }
    fn floatize(q: i8) -> Probability {
        Probability::from(q) / Probability::from(Self::N)
    }
}

/// probability isomorphism
///
/// for river, we use a i8 to represent the equity bucket,
/// i.e. Equity(0) is the 0% equity bucket,
/// and Equity(N) is the 100% equity bucket.
impl From<Probability> for Abstraction {
    fn from(p: Probability) -> Self {
        Self::Equity(Abstraction::quantize(p))
    }
}
impl From<Abstraction> for Probability {
    fn from(abstraction: Abstraction) -> Self {
        match abstraction {
            Abstraction::Equity(n) => Abstraction::floatize(n),
            Abstraction::Random(_) => unreachable!("no cluster into probability"),
        }
    }
}

/// u64 isomorphism
///
/// conversion to u64 for SQL storage.
impl From<Abstraction> for u64 {
    fn from(a: Abstraction) -> Self {
        match a {
            Abstraction::Random(n) => n,
            Abstraction::Equity(_) => unreachable!("no equity into u64"),
        }
    }
}
impl From<u64> for Abstraction {
    fn from(n: u64) -> Self {
        Self::Random(n)
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
        Self::Random(n as u64)
    }
}

impl std::fmt::Display for Abstraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Random(n) => write!(f, "{:016x}", n),
            Self::Equity(_) => unreachable!("don't log me"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_quantize_inverse_floatize() {
        for p in (0..=100).map(|x| x as Probability / 100.0) {
            let q = Abstraction::quantize(p);
            let f = Abstraction::floatize(q);
            assert!((p - f).abs() < 1.0 / Abstraction::N as Probability);
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
}
