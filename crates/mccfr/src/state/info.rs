use crate::*;

/// Information set: what a player knows at a decision point.
///
/// In imperfect-information games, players can't distinguish between
/// certain game states (e.g., can't see opponent's cards). An information
/// set groups all such states—the player must use the same strategy
/// at each state in the set.
///
/// # Components
///
/// An info set combines:
/// - **Public state** (`X: Public`) — observable by all, provides `choices()` and `history()`
/// - **Private state** (`Y: Private`) — observable only by acting player
///
/// # Key Property
///
/// All states in an information set must have the same available actions.
/// CFR computes one strategy per info set.
pub trait CfrInfo:
    Clone + Copy + PartialEq + Eq + Ord + Send + Sync + std::hash::Hash + std::fmt::Debug
{
    /// Edge type for this game.
    type E: CfrEdge;
    /// Turn type for this game.
    type T: CfrTurn;
    /// Public state type.
    type X: CfrPublic<E = Self::E, T = Self::T>;
    /// Private state type.
    type Y: CfrSecret;

    /// Access the public component.
    fn public(&self) -> Self::X;
    /// Access the private component.
    fn secret(&self) -> Self::Y;

    /// Available actions at this decision point.
    fn choices(&self) -> Vec<Self::E> {
        self.public().choices()
    }
    /// Edge history leading to this point (current phase only).
    fn history(&self) -> Vec<Self::E> {
        self.public().history()
    }
}
