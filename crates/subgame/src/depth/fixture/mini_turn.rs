use mccfr::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MiniTurn {
    P0,
    P1,
    Chance,
    Terminal,
}

impl CfrTurn for MiniTurn {
    fn chance() -> Self {
        Self::Chance
    }

    fn terminal() -> Self {
        Self::Terminal
    }

    fn players() -> usize {
        2
    }
}

impl From<usize> for MiniTurn {
    fn from(i: usize) -> Self {
        match i % 2 {
            0 => Self::P0,
            _ => Self::P1,
        }
    }
}
