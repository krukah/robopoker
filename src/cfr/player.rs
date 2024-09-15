use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Player {
    P1,
    P2,
    Chance,
}

#[allow(unused)]
trait CFRPlayer
where
    Self: Sized,
    Self: Clone,
    Self: Copy,
    Self: Hash,
    Self: Eq,
{
}

impl CFRPlayer for Player {}
