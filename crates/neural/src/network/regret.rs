use crate::*;
use mccfr::*;
use pokerkit::*;

/// The advantage network `V(I, a | θ_p)` of Deep CFR, in place of the tabular `cum_regret`.
///
/// `est_regret` returns one entry per `info.choices()`. `see_regret` fills the walker's
/// memory `M_V,p`; `fit_regret` re-initializes `θ_p` and minimizes
/// `Σ t · Σ_a (r̃ᵗ(I,a) − V(I,a|θ_p))²` over it, from scratch every iteration.
pub trait RegretNet: CfrSpec {
    fn est_regret(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Regret>;
    fn see_regret(&mut self, walker: TurnOf<Self>, sample: Sample<Self::F, Regret>);
    fn fit_regret(&mut self, walker: TurnOf<Self>);

    /// Regret matching on predicted advantages. With no positive advantage, Deep CFR plays the
    /// argmax rather than uniform; ties split evenly, so an untrained net plays uniform.
    fn matched(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        let ref regrets = self.est_regret(info);
        let best = regrets
            .iter()
            .map(|&(_, r)| r)
            .fold(Utility::NEG_INFINITY, Utility::max);
        let ties = regrets.iter().filter(|&&(_, r)| r == best).count() as Probability;
        let total = regrets.iter().map(|&(_, r)| r.max(0.)).sum::<Utility>();
        regrets
            .iter()
            .map(|&(e, r)| {
                if total > 0. {
                    (e, r.max(0.) / total)
                } else {
                    (e, Probability::from(r == best) / ties)
                }
            })
            .collect()
    }
}
