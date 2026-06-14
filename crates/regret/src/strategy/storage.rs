use crate::*;
use pokerkit::*;

/// Raw write access layer for CFR strategy data.
///
/// Provides mutable references to accumulated regrets, weights,
/// expected values, and visits. Extends [`CfrRule`] for the
/// shared associated types.
pub trait MutProf: CfrRule {
    /// mutable reference to accumulated weight for this information-action pair
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Probability;
    /// mutable reference to accumulated regret for this information-action pair
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility;
    /// mutable reference to accumulated payoff for this information-action pair
    fn mut_payoff(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility;
    /// mutable reference to encounter visits for this information-action pair
    fn mut_visits(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32;
}
