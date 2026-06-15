//! Depth-limiting phase: base game play or progression through the frontier game.
use super::*;

/// Depth-limiting phase machine.
///
/// Variants are named by *what just happened* to arrive at this state:
/// - `Delegate`: base game play (no frontier entered yet)
/// - `Frontier(payoffs)`: frontier just entered (0 picks; internal to move)
/// - `Internal(payoffs, k)`: internal just picked k (1 pick; external to move)
/// - `External(payoffs, k, j)`: external just picked j (2 picks; resolved)
///
/// `Delegate` stands apart from the frontier variants, but the three
/// frontier variants form a progression. This matches the asymmetry in
/// their semantics: `Delegate` delegates to the inner game, while the
/// frontier variants drive the L×L normal-form subgame.
#[derive(Debug, Clone, Copy)]
pub enum DepthPhase<const D: usize> {
    Delegate,
    Frontier(Payoffs<D>),
    Internal(Payoffs<D>, Continuation),
    External(Payoffs<D>, Continuation, Continuation),
}
