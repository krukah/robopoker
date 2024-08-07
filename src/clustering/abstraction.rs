use super::histogram::Histogram;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// Abstraction represents a lookup value for a given set of Observations.
///
/// - River: we use a u8 to represent the equity bucket, i.e. Equity(0) is the worst bucket, and Equity(50) is the best bucket.
/// - Pre-Flop: we do not use any abstraction, rather store the 169 strategically-unique hands as u64.
/// - Other Streets: we use a u64 to represent the hash signature of the centroid Histogram over lower layers of abstraction.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Abstraction(u64); // hash signature generated by the centroid histogram over lower layers of abstraction

impl Abstraction {
    pub fn buckets() -> Vec<Self> {
        (0..Self::BUCKETS).map(|i| Self(i as u64)).collect()
    }
    pub const BUCKETS: u8 = 50;
}

impl From<&Histogram> for Abstraction {
    fn from(histogram: &Histogram) -> Self {
        let ref mut hasher = DefaultHasher::new();
        histogram.hash(hasher);
        let bucket = hasher.finish();
        Self(bucket)
    }
}

impl From<Abstraction> for u64 {
    fn from(a: Abstraction) -> Self {
        match a {
            Abstraction(n) => n,
        }
    }
}
impl From<u64> for Abstraction {
    fn from(n: u64) -> Self {
        Abstraction(n)
    }
}

/// Conversion to i64 for SQL storage.
impl From<Abstraction> for i64 {
    fn from(abstraction: Abstraction) -> Self {
        u64::from(abstraction) as i64
    }
}
impl From<i64> for Abstraction {
    fn from(n: i64) -> Self {
        Abstraction(n as u64)
    }
}
