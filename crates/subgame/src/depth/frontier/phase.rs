//! Depth-limiting phase: base game play or progression through the frontier game.
use crate::*;

/// Depth-limiting phase machine.
///
/// `Delegate` stands apart from the frontier variants, but the three frontier variants form a
/// progression. This matches the asymmetry in their semantics: `Delegate` delegates to the
/// inner game, while the frontier variants drive the L×L normal-form subgame.
#[derive(Debug, Clone, Copy)]
pub enum DepthPhase<const D: usize> {
    Delegate,
    Frontier(Payoffs<D>),
    Internal(Payoffs<D>, Continuation),
    External(Payoffs<D>, Continuation, Continuation),
}
