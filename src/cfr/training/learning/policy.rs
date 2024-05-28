use crate::cfr::training::marker::action::Action;
use crate::cfr::training::Probability;

/// A policy (P: node -> prob) is a distribution over A(Ii). Easily implemented as a HashMap<Aaction, Probability>.
pub(crate) trait Policy {
    // required
    fn weight(&self, action: &Self::PAction) -> Probability;
    fn sample(&self) -> &Self::PAction;

    type PAction: Action;
}
