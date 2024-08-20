use super::equivalence::Abstraction;

/// A unique identifier for a pair of abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Pair(u64);
impl From<(Abstraction, Abstraction)> for Pair {
    fn from((a, b): (Abstraction, Abstraction)) -> Self {
        Self(u64::from(a) ^ u64::from(b))
    }
}
impl From<Pair> for i64 {
    fn from(pair: Pair) -> Self {
        pair.0 as i64
    }
}
