use crate::*;
use monge::Density;
use pokerkit::*;

/// Core read-only access layer for CFR strategy data.
///
/// Provides the required methods for reading accumulated regrets, weights,
/// expected values, and visits. All methods take `&self` for read-only queries.
/// Decoupled from `Storage` (the write layer) so that read-only consumers
/// need not carry mutable access. Extends [`CfrRule`] for the shared
/// associated types.
pub trait RefProf: CfrRule {
    /// how many iterations
    fn t(&self) -> usize;
    /// Sum of positive regrets across all infosets, divided by iterations.
    fn sum_regret(&self) -> Utility;
    /// lookup accumulated weight for this information
    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability;
    /// lookup accumulated regret for this information
    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility;
    /// lookup accumulated payoff for this information-action pair
    fn cum_payoff(&self, info: &Self::I, edge: &Self::E) -> Utility;
    /// lookup accumulated encounter visits for this information-action pair
    fn cum_visits(&self, info: &Self::I, edge: &Self::E) -> u32;

    /// optional metrics for logging (default: None)
    fn metrics(&self) -> Option<&Metrics> {
        None
    }
    /// floored accumulated regret (never below POLICY_MIN)
    fn regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.cum_regret(info, edge).max(EPSILON)
    }
    /// floored accumulated weight (never below POLICY_MIN)
    fn weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.cum_weight(info, edge).max(EPSILON)
    }
    /// Historical weighted average strategy (Nash approximation).
    /// Derived from accumulated weights. Override to apply perturbation.
    fn averaged_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let all = info.choices().map(|e| (e, self.weight(info, &e))).collect::<Vec<_>>();
        let sum = all.iter().map(|(_, w)| *w).sum::<Probability>();
        all.into_iter().map(|(e, w)| (e, w / sum)).collect()
    }
    /// Current iteration strategy via regret matching.
    /// Derived from accumulated regrets.
    fn iterated_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let raw = info.choices().map(|e| (e, self.regret(info, &e))).collect::<Vec<_>>();
        let denom = raw.iter().map(|(_, r)| r).sum::<Utility>();
        raw.into_iter().map(|(e, r)| (e, r / denom)).collect()
    }

    /// Seed a subgame-local CFR accumulator from this profile's accumulated
    /// values, in a way that's agnostic to how long this profile has been
    /// trained.
    ///
    /// # Why iteration-count agnosticism matters
    ///
    /// Under `LinearWeight`, `cum_weight` grows as O(T²) in the blueprint's
    /// iteration count T. The naive `(K/T) × cum_weight` scaling is linear
    /// in T despite the division — a 12M-iter blueprint contributes ~400×
    /// more warmstart weight than a 30k-iter one for the same K. At deep
    /// T, subgame's own 50k iterations are drowned by the prior and the
    /// refined policy collapses to blueprint. This is not a well-behaved
    /// knob.
    ///
    /// This implementation synthesizes warmstart weight as if the local
    /// profile had run `K = WarmstartHyperParams::prior_strength()`
    /// iterations of `LinearWeight` with the blueprint's averaged policy:
    ///
    /// ```text
    /// cum_weight ≈ policy × K(K+1)/2  (LinearWeight closed-form for constant policy)
    /// ```
    ///
    /// This is independent of `self.t()`. K becomes a genuine
    /// "effective iteration count for the prior," directly comparable to
    /// the subgame's own ~50k iterations regardless of blueprint depth.
    ///
    /// `regret` still uses the simple `K/T` scaling because `LinearRegret`
    /// already converges to a bounded value (time-weighted mean of
    /// immediate regrets), so the naive formula doesn't have the quadratic
    /// blow-up problem. For a converged blueprint, warmstart regret is
    /// near-zero regardless of scaling.
    ///
    /// `payoff` and `visits` are reset to zero:
    ///
    /// - `payoff` is measured against this profile's tree; in
    ///   depth-limited subgame variants the subgame tree terminates at a
    ///   biased-rollout frontier, a different metric space than real
    ///   terminals. Inheriting blueprint payoff injects incompatible
    ///   samples.
    /// - `visits` gates the extraction-time blend weight; synthetic
    ///   "training visits" bias the blend.
    fn warmstart(&self, info: &Self::I, edge: &Self::E) -> Encounter {
        let k = WarmstartHyperParams::get().prior_strength() as Utility;
        let policy = self.averaged_distribution(info).density(edge);
        let regret_scale = k / self.t().max(1) as Utility;
        Encounter {
            weight: policy * k * (k + 1.0) / 2.0,
            regret: self.cum_regret(info, edge) * regret_scale,
            payoff: 0.0,
            visits: 0,
        }
    }
}
