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
