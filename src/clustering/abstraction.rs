use crate::cards::street::Street;
use crate::transport::support::Support;
use crate::Arbitrary;
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
    Percent(u64), // river
    Learned(u64), // flop, turn
    Preflop(u64), // preflop
}

impl Abstraction {
    const N: usize = crate::KMEANS_EQTY_CLUSTER_COUNT - 1;
    pub const fn size() -> usize {
        Self::N as usize + 1
    }
    pub fn range() -> impl Iterator<Item = Self> {
        (0..=Self::N).map(|i| Self::from((Street::Rive, i as usize)))
    }
    pub fn street(&self) -> Street {
        match self {
            Abstraction::Percent(n) | Abstraction::Learned(n) | Abstraction::Preflop(n) => {
                match (n >> LSB) as isize {
                    0 => Street::Pref,
                    1 => Street::Flop,
                    2 => Street::Turn,
                    3 => Street::Rive,
                    _ => panic!("at the disco"),
                }
            }
        }
    }
    pub fn index(&self) -> usize {
        match self {
            Abstraction::Percent(n) | Abstraction::Learned(n) | Abstraction::Preflop(n) => {
                (n & ((1 << LSB) - 1)) as usize
            }
        }
    }

    fn quantize(p: Probability) -> usize {
        (p * Self::N as Probability).round() as usize
    }
    fn floatize(q: usize) -> Probability {
        q as Probability / Self::N as Probability
    }
}

impl From<(Street, usize)> for Abstraction {
    fn from((street, index): (Street, usize)) -> Self {
        match street {
            Street::Pref => Abstraction::Preflop(((street as u8 as u64) << LSB) | (index as u64)),
            Street::Flop => Abstraction::Learned(((street as u8 as u64) << LSB) | (index as u64)),
            Street::Turn => Abstraction::Learned(((street as u8 as u64) << LSB) | (index as u64)),
            Street::Rive => Abstraction::Percent(((street as u8 as u64) << LSB) | (index as u64)),
        }
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
        Self::from((Street::Rive, Self::quantize(p)))
    }
}
impl From<Abstraction> for Probability {
    fn from(abstraction: Abstraction) -> Self {
        match abstraction {
            Abstraction::Percent(_) => Abstraction::floatize(abstraction.index()),
            Abstraction::Learned(_) => unreachable!("no cluster into probability"),
            Abstraction::Preflop(_) => unreachable!("no preflop into probability"),
        }
    }
}

/// u64 isomorphism
///
/// conversion to u64 for SQL storage.
impl From<Abstraction> for u64 {
    fn from(a: Abstraction) -> Self {
        // let street = a.street();
        // let index = a.index();
        // let bits = ((street as u8 as u64) << LSB) | (index as u64);
        // bits
        match a {
            Abstraction::Percent(n) | Abstraction::Learned(n) | Abstraction::Preflop(n) => n,
        }
    }
}
impl From<u64> for Abstraction {
    fn from(n: u64) -> Self {
        match (n >> LSB) as u8 {
            0 => Abstraction::Preflop(n),
            1 => Abstraction::Learned(n),
            2 => Abstraction::Learned(n),
            3 => Abstraction::Percent(n),
            _ => unreachable!("Invalid street value"),
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
        Self::from(n as u64)
    }
}
/// string isomorophism
impl TryFrom<&str> for Abstraction {
    type Error = Box<dyn std::error::Error>;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.trim().split("::").collect::<Vec<_>>();
        let a = s[0];
        let b = s[1];
        let street = Street::try_from(a)?;
        let index = usize::from_str_radix(b, 16)?;
        Ok(Abstraction::from((street, index)))
    }
}
impl std::fmt::Display for Abstraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{:02x}",
            self.street()
                .to_string()
                .chars()
                .next()
                .unwrap()
                .to_uppercase(),
            self.index()
        )
    }
}

impl Arbitrary for Abstraction {
    fn random() -> Self {
        use rand::Rng;
        let street = Street::random();
        let n = street.k();
        let i = rand::thread_rng().gen_range(0..n);
        Abstraction::from((street, i))
    }
}

impl Support for Abstraction {}

/// establish that the lower 56 LSBs are what the index
/// contains information about Abstraction
/// the upper 8 MSBs is the Street tag'
const LSB: usize = 56;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::observation::Observation;
    use crate::cards::street::Street;
    use crate::Arbitrary;

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
        let equity = Abstraction::from(Observation::from(Street::Rive).equity());
        assert_eq!(equity, Abstraction::from(u64::from(equity)));
    }
}
