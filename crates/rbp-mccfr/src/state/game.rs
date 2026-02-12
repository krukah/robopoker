use crate::*;
use rbp_core::Utility;

/// The memoryless game state for CFR traversal.
///
/// Implements the core game logic: state transitions via actions and
/// payoff computation at terminal nodes. The state should be minimal
/// and local—history tracking belongs in [`TreeInfo`] or [`Encoder`].
///
/// # Required Methods
///
/// - `root()` — Creates the starting game state
/// - `turn()` — Returns whose turn it is (player/chance/terminal)
/// - `apply(edge)` — Returns new state after taking an action
/// - `payoff(turn)` — Returns utility for a player at terminal nodes
///
/// # Design Notes
///
/// Games must be `Copy` for efficient tree traversal. Complex state
/// should use compact representations (bitboards, packed integers).
pub trait CfrGame: Clone + Copy + Send + Sync {
    type E: CfrEdge;
    type T: CfrTurn;
    fn root() -> Self;
    fn turn(&self) -> Self::T;
    fn apply(&self, edge: Self::E) -> Self;
    fn payoff(&self, turn: Self::T) -> Utility;
}
