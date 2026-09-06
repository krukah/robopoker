use crate::*;
use pokerkit::Utility;

/// The memoryless game state for CFR traversal: transitions via actions and
/// payoffs at terminal nodes.
///
/// Keep implementations minimal and local — history tracking belongs in
/// [`CfrInfo`] or [`CfrEncoder`]. The `Copy` bound means complex state should
/// use compact representations (bitboards, packed integers).
pub trait CfrGame: Clone + Copy + Send + Sync {
    type E: CfrEdge;
    type T: CfrTurn;

    fn root() -> Self;
    fn turn(&self) -> Self::T;
    fn apply(&self, edge: Self::E) -> Self;
    fn payoff(&self, turn: Self::T) -> Utility;

    /// Coarse depth indicator for street-level boundary detection.
    ///
    /// Override for games with distinct phases (e.g. poker streets) where
    /// subgame solving should stop at phase boundaries rather than expanding
    /// the full remaining tree.
    fn depth(&self) -> usize {
        0
    }
    /// True if this chance node should expand into continuation choices rather
    /// than be treated as a leaf — how encoders spot frontier nodes.
    ///
    /// False by default (chance nodes are leaves in depth-limited solving);
    /// `FrontGame` overrides it at the depth limit.
    fn is_frontier(&self) -> bool {
        false
    }
    /// Root node for exploitability computation.
    ///
    /// Override for games needing a different starting state — e.g. a chance
    /// node that deals all possible hands.
    fn exploitability_root() -> Self {
        Self::root()
    }
}
