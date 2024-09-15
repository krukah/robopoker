use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(usize);

impl Bucket {
    pub const IGNORE: Self = Self(0);
    pub const P1: Self = Self(1);
    pub const P2: Self = Self(2);
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
