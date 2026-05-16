use crate::*;
use rbp_core::Utility;

/// The memoryless game state for CFR traversal.
///
/// Implements the core game logic: state transitions via actions and
/// payoff computation at terminal nodes. The state should be minimal
/// and local—history tracking belongs in [`CfrInfo`] or [`CfrEncoder`].
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
    /// Coarse depth indicator for street-level boundary detection.
    ///
    /// Returns 0 by default. Override for games with distinct phases
    /// (e.g., poker streets) where subgame solving should stop at
    /// phase boundaries rather than expanding the full remaining tree.
    fn depth(&self) -> usize {
        0
    }
    /// True if this is a chance node that should expand into continuation
    /// choices rather than being treated as a leaf. Used by encoders to
    /// distinguish frontier nodes from regular chance nodes.
    ///
    /// Default: false (all chance nodes are leaves in depth-limited solving).
    /// `FrontGame` overrides this to return true at the depth limit.
    fn is_frontier(&self) -> bool {
        false
    }
    /// Root node for exploitability computation.
    ///
    /// Defaults to [`Self::root()`]. Override for games where the
    /// exploitability tree requires a different starting state
    /// (e.g., a chance node that deals all possible hands).
    fn exploitability_root() -> Self {
        Self::root()
    }
}
