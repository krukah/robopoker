use crate::cfr::traits::turn::Turn;
use crate::Utility;

/// Represents a game state
pub trait Game: Clone + Copy {
    type T: Turn;
    fn root() -> Self;
    fn turn(&self) -> Self::T;
    fn payoff(&self, player: Self::T) -> Utility;
}

impl Game for crate::gameplay::game::Game {
    type T = crate::gameplay::ply::Turn;
    fn turn(&self) -> Self::T {
        self.turn()
    }
    fn root() -> Self {
        Self::root()
    }
    fn payoff(&self, turn: Self::T) -> Utility {
        self.settlements()
            .get(turn.player())
            .map(|settlement| settlement.pnl() as f32)
            .expect("player index in bounds")
    }
}
