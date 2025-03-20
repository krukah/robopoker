use super::edge::{Decision, Turn};
use crate::cfr::traits::turn::Playee;
use crate::Utility;

/// Represents a game state
pub trait Game: Clone + Copy {
    type E: Turn;
    type W: Playee;
    fn root() -> Self;
    fn turn(&self) -> Self::W;
    fn payoff(&self, player: Self::W) -> Utility;
}

impl Game for crate::gameplay::game::Game {
    type E = crate::gameplay::action::Action;
    type W = crate::gameplay::ply::Turn;
    fn root() -> Self {
        Self::root()
    }
    fn turn(&self) -> Self::W {
        self.turn()
    }
    fn payoff(&self, turn: Self::W) -> Utility {
        self.settlements()
            .get(turn.player())
            .map(|settlement| settlement.pnl() as f32)
            .expect("player index in bounds")
    }
}
