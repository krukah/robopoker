use crate::play::action::Action;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Edge(Action);

impl From<Action> for Edge {
    fn from(action: Action) -> Self {
        Self(action)
    }
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
