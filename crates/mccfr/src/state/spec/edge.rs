use monge::Support;

/// An action or transition in the game tree: a player decision (bet, fold) or a
/// chance outcome (card deal).
///
/// The bounds make edges usable as strategy-table keys, deterministically
/// iterable, and transportable across threads.
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
    /// Default initial policy weight for CFR warmstart (0.0 = uniform).
    fn default_policy(&self) -> pokerkit::Probability {
        0.0
    }
    /// Default initial regret for CFR warmstart (0.0 = no bias).
    fn default_regret(&self) -> pokerkit::Utility {
        0.0
    }
}
