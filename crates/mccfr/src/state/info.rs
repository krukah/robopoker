use crate::*;

/// What a player knows at a decision point: public state `X` (observable by
/// all) paired with private state `Y` (visible only to the actor).
///
/// Groups the game states a player can't distinguish, so CFR computes exactly
/// one strategy per info set. Every state in a set must therefore offer the
/// same available actions.
pub trait CfrInfo: Clone + Copy + PartialEq + Eq + Ord + Send + Sync + std::hash::Hash + std::fmt::Debug {
    type E: CfrEdge;
    type T: CfrTurn;
    type Y: CfrSecret;
    type X: CfrPublic<E = Self::E, T = Self::T>;

    fn public(&self) -> Self::X;
    fn secret(&self) -> Self::Y;

    /// Available actions at this decision point.
    fn choices(&self) -> impl Iterator<Item = Self::E> + use<Self> {
        self.public().choices()
    }
    /// Edge history leading to this point (current phase only).
    fn history(&self) -> Vec<Self::E> {
        self.public().subgame()
    }
}
