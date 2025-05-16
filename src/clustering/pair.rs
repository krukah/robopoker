use crate::gameplay::abstraction::Abstraction;

/// A unique identifier for a pair of abstractions.
#[derive(Default, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct Pair(u64);

impl From<(&Abstraction, &Abstraction)> for Pair {
    fn from((a, b): (&Abstraction, &Abstraction)) -> Self {
        Self(u64::from(*a) ^ u64::from(*b))
    }
}
impl From<Pair> for i64 {
    fn from(pair: Pair) -> Self {
        pair.0 as i64
    }
}
impl From<i64> for Pair {
    fn from(i: i64) -> Self {
        Self(i as u64)
    }
}
