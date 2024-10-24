use crate::play::transition::Transition;
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

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
