/// A player or node type in the game tree.
///
/// CFR distinguishes three node types: player decisions, chance nodes,
/// and terminal nodes. This trait provides the minimal interface for
/// identifying node types that require different treatment during traversal.
///
/// # Requirements
///
/// - Must have distinguished variants for chance, terminal, and player nodes
/// - Must be cheaply copyable for tree traversal
/// - Must be hashable for use in strategy tables
///
/// # Node Types
///
/// - `chance()` — Random events (card deals, dice rolls)
/// - `terminal()` — Game over states with payoffs
/// - `From<usize>` — Player index to turn (0 = P1, 1 = P2, panics otherwise)
pub trait CfrTurn:
    Clone + Copy + PartialEq + Eq + Send + Sync + std::fmt::Debug + std::hash::Hash + From<usize>
{
    /// Returns the chance node variant.
    fn chance() -> Self;
    /// Returns the terminal node variant.
    fn terminal() -> Self;

    fn is_chance(&self) -> bool {
        &Self::chance() == self
    }
}
