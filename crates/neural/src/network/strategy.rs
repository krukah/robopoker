use crate::*;
use mccfr::*;

/// The average-strategy network `Π(I, a | φ)` of Deep CFR, in place of the tabular `cum_weight`.
///
/// `see_strategy` fills `M_Π` with `σᵗ` at the non-walker's infosets, whose on-policy reach
/// weights them as the average strategy requires. `fit_strategy` minimizes
/// `Σ t · Σ_a (σᵗ(I,a) − Π(I,a|φ))²` once, after the last iteration.
pub trait StrategyNet: CfrSpec {
    fn est_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy>;
    fn see_strategy(&mut self, sample: Sample<Self::F, Policy>);
    fn fit_strategy(&mut self);
}
