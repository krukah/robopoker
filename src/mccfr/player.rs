use crate::gameplay::ply::Ply;
use std::hash::Hash;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Player(pub Ply);

impl Player {
    pub const fn chance() -> Self {
        Self(Ply::Chance)
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Ply::Chance => write!(f, "??"),
            Ply::Choice(0) => write!(f, "P0"),
            Ply::Choice(_) => write!(f, "P1"),
            Ply::Terminal => write!(f, "END"),
        }
    }
}
