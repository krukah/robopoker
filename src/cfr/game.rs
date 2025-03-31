use super::edge::Edge;
use super::turn::Turn;
use crate::Utility;

/// the tree-local game state is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to create a Game from:
///  scratch (i.e. root node without context)
///
/// the implementation must be able to determine:
///  whose turn is it (have a Player function)
///  how much payoff for each player (only must be defined for leaf nodes)
///
/// it is up to the implementation of Encoder to decide how the
/// game tree is navigated, in a tree-non-local context. this Game
/// structure should only concern itself of local properties.
pub trait Game: Clone + Copy {
    type E: Edge;
    type T: Turn;
    fn root() -> Self;
    fn turn(&self) -> Self::T;
    fn apply(&self, edge: Self::E) -> Self;
    fn payoff(&self, turn: Self::T) -> crate::Utility;
}
impl Game for crate::gameplay::game::Game {
    type E = crate::gameplay::action::Action;
    type T = crate::gameplay::ply::Turn;
    fn root() -> Self {
        Self::root()
    }
    fn turn(&self) -> Self::T {
        self.turn()
    }
    fn apply(&self, edge: Self::E) -> Self {
        self.apply(edge)
    }
    fn payoff(&self, turn: Self::T) -> Utility {
        self.settlements()
            .get(turn.player())
            .map(|settlement| settlement.pnl() as f32)
            .expect("player index in bounds")
    }
}
