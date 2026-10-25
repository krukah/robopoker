use crate::*;
use monge::Density;
use pokerkit::*;

/// Read-only access to accumulated regrets, weights, payoffs, and visits.
pub trait Profile: CfrSpec {
    /// how many iterations
    fn t(&self) -> usize;

    /// Sum of positive regrets across all infosets, divided by iterations.
    fn sum_regret(&self) -> Utility;

    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability;
    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility;
    fn cum_payoff(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility;
    fn cum_visits(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> u32;

    /// Optional metrics sink for logging.
    fn metrics(&self) -> Option<&Metrics> {
        None
    }
    /// Accumulated regret floored at `EPSILON`, as regret matching requires.
    fn regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.cum_regret(info, edge).max(EPSILON)
    }
    /// Accumulated weight floored at `EPSILON`.
    fn weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.cum_weight(info, edge).max(EPSILON)
    }
    /// Historical weighted average strategy (Nash approximation). Override to apply
    /// perturbation.
    fn averaged_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        let all = info.menu().map(|e| (e, self.weight(info, &e))).collect::<Vec<_>>();
        let sum = all.iter().map(|(_, w)| *w).sum::<Probability>();
        all.into_iter().map(|(e, w)| (e, w / sum)).collect()
    }
    /// Current iteration strategy via regret matching.
    fn iterated_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        let raw = info.menu().map(|e| (e, self.regret(info, &e))).collect::<Vec<_>>();
        let denom = raw.iter().map(|(_, r)| r).sum::<Utility>();
        raw.into_iter().map(|(e, r)| (e, r / denom)).collect()
    }

    /// Seed a subgame-local CFR accumulator from this profile, agnostic to how long this
    /// profile has been trained.
    fn warmstart(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Encounter {
        let k = WarmstartHyperParams::get().prior_strength() as Utility;
        let policy = self.averaged_strategy(info).density(edge);
        let regret_scale = k / self.t().max(1) as Utility;
        Encounter::new(policy * k * (k + 1.0) / 2.0, self.cum_regret(info, edge) * regret_scale, 0.0, 0)
    }
}
