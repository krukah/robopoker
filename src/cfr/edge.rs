use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Edge {
    RO,
    PA,
    SC,
}

#[allow(unused)]
trait CFREdge
where
    Self: Sized,
    Self: Clone,
    Self: Copy,
    Self: Hash,
    Self: Eq,
    Self: PartialEq,
    Self: Ord,
    Self: PartialOrd,
{
}

impl CFREdge for Edge {}
