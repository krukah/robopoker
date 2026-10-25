use mccfr::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ToyTurn {
    P0,
    P1,
    Chance,
    Terminal,
}

impl CfrTurn for ToyTurn {
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

impl From<usize> for ToyTurn {
    fn from(i: usize) -> Self {
        if i.is_multiple_of(2) { Self::P0 } else { Self::P1 }
    }
}
