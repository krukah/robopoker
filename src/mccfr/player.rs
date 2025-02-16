use crate::gameplay::ply::Next;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Player(pub Next);

impl Player {
    pub const fn chance() -> Self {
        Self(Next::Chance)
    }
}

impl Default for Player {
    fn default() -> Self {
        Self(Next::Choice(0))
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Next::Chance => write!(f, "??"),
            Next::Choice(0) => write!(f, "P0"),
            Next::Choice(_) => write!(f, "P1"),
            Next::Terminal => write!(f, "END"),
        }
    }
}
