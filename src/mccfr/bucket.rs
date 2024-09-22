use crate::clustering::explorer::Abstraction;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(Abstraction);

impl From<Abstraction> for Bucket {
    fn from(abstraction: Abstraction) -> Self {
        Self(abstraction)
    }
}

#[allow(unused)]
trait CFRBucket
where
    Self: Sized,
    Self: Clone,
    Self: Copy,
    Self: Hash,
    Self: Ord,
    Self: Eq,
    Self: PartialOrd,
    Self: PartialEq,
{
}

impl CFRBucket for Bucket {}
