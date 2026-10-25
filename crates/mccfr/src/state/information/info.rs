use crate::*;

/// What a player knows at a decision point: public state `X` (observable by
/// all) paired with private state `Y` (visible only to the actor).
///
/// Groups the game states a player can't distinguish, so CFR computes exactly
/// one strategy per info set. Every state in a set must therefore offer the
/// same available actions.
pub trait CfrInfo: Clone + Copy + PartialEq + Eq + Ord + Send + Sync + std::hash::Hash + std::fmt::Debug {
    type Y: CfrSecret;
    type X: CfrPublic;

    fn public(&self) -> Self::X;
    fn secret(&self) -> Self::Y;
    /// The info set with these halves: the inverse of `(public, secret)`.
    fn join(public: Self::X, secret: Self::Y) -> Self;

    /// Available actions at this decision point.
    fn menu(&self) -> impl Iterator<Item = EdgeOf<Self>> + use<Self> {
        self.public().menu()
    }
}
