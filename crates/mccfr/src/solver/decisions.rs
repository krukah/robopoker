use crate::*;
use pokerkit::Utility;

/// The minimal update unit for CFR profile learning: one infoset's per-action
/// `regret` increments, the `policy` used this iteration, and the infoset EV.
///
/// [`Solution`] folds these into cumulative regrets and average strategy;
/// `payoff` supplies the frontier values depth-limited search and safe subgame
/// solving need.
#[derive(Clone)]
pub struct Decisions<F>
where
    F: CfrInfo,
{
    info: F,
    regret: Over<EdgeOf<F>, Regret>,
    policy: Over<EdgeOf<F>, Policy>,
    payoff: Utility,
}

impl<F> Decisions<F>
where
    F: CfrInfo,
{
    pub fn new(info: F, regret: Over<EdgeOf<F>, Regret>, policy: Over<EdgeOf<F>, Policy>, payoff: Utility) -> Self {
        Self {
            info,
            regret,
            policy,
            payoff,
        }
    }

    pub fn info(&self) -> &F {
        &self.info
    }

    pub fn regret(&self) -> &Over<EdgeOf<F>, Regret> {
        &self.regret
    }

    pub fn policy(&self) -> &Over<EdgeOf<F>, Policy> {
        &self.policy
    }

    pub fn payoff(&self) -> Utility {
        self.payoff
    }
}
