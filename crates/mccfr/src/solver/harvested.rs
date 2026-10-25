use pokerkit::Probability;
use pokerkit::Utility;
use std::collections::BTreeMap;

/// Refined per-edge policy, visit counts, and total positive regret at the
/// harvested infoset. `regret` is `Σ_a max(0, R(I, a))` summed over choice
/// edges (and over partitions for world / subgame variants), matching how
/// `visits` aggregates. Divide by the sum of `visits` for per-iteration
/// regret at this decision.
pub struct Harvested<E> {
    refined: BTreeMap<E, Probability>,
    visits: BTreeMap<E, u32>,
    regret: Utility,
}
impl<E> Harvested<E> {
    pub fn new(refined: BTreeMap<E, Probability>, visits: BTreeMap<E, u32>, regret: Utility) -> Self {
        Self {
            refined,
            visits,
            regret,
        }
    }

    pub fn refined(&self) -> &BTreeMap<E, Probability> {
        &self.refined
    }

    pub fn visits(&self) -> &BTreeMap<E, u32> {
        &self.visits
    }

    pub fn regret(&self) -> Utility {
        self.regret
    }
}
