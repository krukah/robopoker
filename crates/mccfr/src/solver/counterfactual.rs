use crate::*;
use rbp_core::Utility;

/// The minimal update unit for CFR profile learning.
///
/// Bundles an information set identifier with policies and expected value:
/// 1. **Regret policy** — Counterfactual regret increments per action
/// 2. **Strategy policy** — Current iteration's action probabilities
/// 3. **Infoset EV** — Expected value of the information set this iteration
///
/// The [`Profile`] accumulates these updates to maintain cumulative
/// regrets, average strategy, and expected values over training iterations.
///
/// # Fields
///
/// - `info` — Information set identifier
/// - `regret` — Regret policy: how much better each action would have been
/// - `policy` — Strategy policy: probabilities used this iteration
/// - `evalue` — Expected value of the infoset under current strategy
///
/// The `evalue` field enables depth-limited search and safe subgame solving
/// by providing frontier evaluation values.
pub struct Counterfactual<E, I>
where
    E: CfrEdge,
    I: CfrInfo<E = E>,
{
    pub info: I,
    pub regret: Policy<E>,
    pub policy: Policy<E>,
    pub evalue: Utility,
}
