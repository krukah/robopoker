use crate::play::continuation::Transition;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Player {
    Choice(Transition),
    Chance,
}

impl Player {
    pub const fn chance() -> Self {
        Self::Chance
    }
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
