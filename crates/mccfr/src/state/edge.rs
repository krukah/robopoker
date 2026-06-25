use rbp_transport::Support;

/// An action or transition in the game tree.
///
/// Edges represent decisions that players can make (bet, fold, etc.) or
/// chance outcomes (card deals). The trait bounds ensure edges can be
/// used as keys in strategy tables and transported across threads.
///
/// # Requirements
///
/// - Copyable and hashable for strategy table lookups
/// - Ordered for deterministic iteration
/// - Implements [`Support`](rbp_transport::Support) for probability distributions
pub trait CfrEdge:
    Copy
    + Clone
    + PartialEq
    + Eq
    + PartialOrd // can be ignored
    + Ord // can be ignored
    + Send
    + Sync
    + Support
    + std::hash::Hash // can be ignored
    + std::fmt::Debug
{
    /// True for fold-like player actions.
    fn is_fold(&self) -> bool {
        false
    }
    /// True for call-like player actions.
    fn is_call(&self) -> bool {
        false
    }
    /// True for aggressive player actions (bets, raises, shoves).
    fn is_raise(&self) -> bool {
        false
    }
}
