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

/// establish that the lower 56 LSBs are what the index
/// contains information about Abstraction
/// the upper 8 MSBs is the Street tag'

const H: u64 = 0xFF00000000000000; // street mask
const M: u64 = 0x00FFFFFFFFFFF000; // hash mask
const L: u64 = 0x0000000000000FFF; // index mask

impl Abstraction {
    const DELIM: &'static str = "::";
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
                match ((H & n) >> H.count_zeros()) as isize {
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
                (L & n) as usize
            }
        }
    }
    pub fn all(street: Street) -> Vec<Self> {
        if street == Street::Rive {
            Self::range().collect()
        } else {
            (0..street.k()).map(|i| Self::from((street, i))).collect()
        }
    }
    fn signature(street: Street, index: usize) -> usize {
        let bits = L & index as u64;
        let bits = bits | (street as u8 as u64) << L.count_ones();
        let bits = bits.wrapping_mul(0x9E3779B97F4A7C15);
        let bits = M & bits;
        bits as usize
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
        let mut bits = 0;
        bits |= L & index as u64;
        bits |= M & Self::signature(street, index) as u64;
        bits |= H & (street as u8 as u64) << H.count_zeros();
        match street {
            Street::Pref => Abstraction::Preflop(bits),
            Street::Flop => Abstraction::Learned(bits),
            Street::Turn => Abstraction::Learned(bits),
            Street::Rive => Abstraction::Percent(bits),
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
        match a {
            Abstraction::Percent(n) | Abstraction::Learned(n) | Abstraction::Preflop(n) => n,
        }
    }
}
impl From<u64> for Abstraction {
    fn from(n: u64) -> Self {
        match ((H & n) >> H.count_zeros()) as u8 {
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
        let s = s.trim().split(Self::DELIM).collect::<Vec<_>>();
        let a = s.get(0).copied().ok_or("broken delimiter")?;
        let b = s.get(1).copied().ok_or("broken delimiter")?;
        let street = Street::try_from(a)?;
        let index = usize::from_str_radix(b, 16)?;
        Ok(Abstraction::from((street, index)))
    }
}
impl std::fmt::Display for Abstraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{:02x}",
            self.street()
                .to_string()
                .chars()
                .next()
                .unwrap()
                .to_uppercase(),
            Self::DELIM,
            self.index()
        )
    }
}

impl Arbitrary for Abstraction {
    fn random() -> Self {
        use rand::Rng;
        let street = Street::Flop;
        let k = street.k();
        let i = rand::thread_rng().gen_range(0..k);
        Abstraction::from((street, i))
    }
}

impl Support for Abstraction {}

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
    #[test]
    fn bijective_str() {
        let abs = Abstraction::random();
        let str = format!("{}", abs);
        assert_eq!(abs, Abstraction::try_from(str.as_str()).unwrap());
    }
}
