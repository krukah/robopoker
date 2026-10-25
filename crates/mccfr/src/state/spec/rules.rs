use crate::*;

/// The pure game: exact moves and deterministic transitions, enough to generate consistent
/// hands. A [`CfrGame`] whose edges are already lossless is its own pure game; one whose edges
/// forget something pairs a `Rules` with an [`Approximation`].
pub trait Rules: Copy {
    /// Chip-exact and chance-resolved.
    type Move: Copy;
    type Turn: CfrTurn;

    fn turn(&self) -> Self::Turn;
    fn apply(&self, m: Self::Move) -> Self;
}
