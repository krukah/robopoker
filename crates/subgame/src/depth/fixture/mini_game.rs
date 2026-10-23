use super::*;
use mccfr::*;
use pokerkit::*;

#[derive(Debug, Clone, Copy)]
pub struct MiniGame {
    street: usize,
    actions: usize,
}

impl CfrGame for MiniGame {
    type E = MiniEdge;
    type T = MiniTurn;

    fn root() -> Self {
        Self { street: 0, actions: 0 }
    }

    fn turn(&self) -> Self::T {
        if self.street() >= 2 {
            MiniTurn::Terminal
        } else if self.actions() >= 2 {
            MiniTurn::Chance
        } else if self.actions().is_multiple_of(2) {
            MiniTurn::P0
        } else {
            MiniTurn::P1
        }
    }

    fn apply(&self, _edge: Self::E) -> Self {
        if self.turn() == MiniTurn::Chance {
            Self {
                street: self.street() + 1,
                actions: 0,
            }
        } else {
            Self {
                street: self.street(),
                actions: self.actions() + 1,
            }
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        match turn {
            MiniTurn::P0 => 1.0,
            MiniTurn::P1 => -1.0,
            _ => 0.0,
        }
    }

    fn depth(&self) -> usize {
        if self.turn() == MiniTurn::Chance { self.street() + 1 } else { self.street() }
    }
}

impl MiniGame {
    pub fn street(&self) -> usize {
        self.street
    }

    pub fn actions(&self) -> usize {
        self.actions
    }
}
