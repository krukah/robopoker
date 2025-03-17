use crate::cfr::traits::turn::Turn;

/// Represents a game state
pub trait Game: Clone + Copy {
    type T: Turn;
    fn root() -> Self;
    fn turn(&self) -> Self::T;
}

impl Game for crate::gameplay::game::Game {
    type T = crate::gameplay::ply::Turn;
    fn turn(&self) -> Self::T {
        self.turn()
    }
    fn root() -> Self {
        Self::root()
    }
}
