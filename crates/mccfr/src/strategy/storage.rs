use crate::*;
use pokerkit::*;

/// Write access to CFR strategy data: mutable references to the accumulated
/// weight, regret, payoff, and visits of one info-action pair.
pub trait MutProf: CfrRule {
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Probability;
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility;
    fn mut_payoff(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility;
    fn mut_visits(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32;
}
