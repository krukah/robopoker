use crate::clustering::abstraction::CardAbstraction;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(CardAbstraction);

impl From<CardAbstraction> for Bucket {
    fn from(abstraction: CardAbstraction) -> Self {
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
