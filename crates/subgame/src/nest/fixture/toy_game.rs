use super::*;
use crate::*;
use mccfr::*;
use pokerkit::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToyGame {
    pot: i32,
}

impl CfrGame for ToyGame {
    type E = ToyEdge;
    type T = ToyTurn;

    fn root() -> Self {
        Self { pot: 1 }
    }

    fn turn(&self) -> Self::T {
        ToyTurn::P0
    }

    fn apply(&self, edge: Self::E) -> Self {
        Self {
            pot: self.pot() + i32::from(matches!(edge, ToyEdge::Bet)),
        }
    }

    fn payoff(&self, _: Self::T) -> Utility {
        0.0
    }
}

impl Augmentable for ToyGame {
    type Off = i32;

    fn augment(&self, off: Self::Off) -> Self {
        Self { pot: self.pot() + off }
    }
}

impl ToyGame {
    pub fn pot(&self) -> i32 {
        self.pot
    }
}
