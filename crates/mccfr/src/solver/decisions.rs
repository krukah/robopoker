use crate::*;
use pokerkit::Utility;

/// The minimal update unit for CFR profile learning: one infoset's per-action
/// `regret` increments, the `policy` used this iteration, and the infoset EV.
///
/// [`CfrSolution`] folds these into cumulative regrets and average strategy;
/// `payoff` supplies the frontier values depth-limited search and safe subgame
/// solving need.
pub struct Decisions<E, I>
where
    E: CfrEdge,
    I: CfrInfo<E = E>,
{
    pub info: I,
    pub regret: Policy<E>,
    pub policy: Policy<E>,
    pub payoff: Utility,
}
