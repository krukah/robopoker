use crate::*;
use pokerkit::*;

/// Write access to CFR strategy data: mutable references to the accumulated
/// weight, regret, payoff, and visits of one info-action pair.
pub trait Storage: CfrSpec {
    fn mut_weight(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Probability;
    fn mut_regret(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Utility;
    fn mut_payoff(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Utility;
    fn mut_visits(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut u32;
}
