use crate::cfr::traits::marker::action::Action;
use crate::cfr::traits::Probability;

/// A policy (P: node -> prob) is a distribution over A(Ii). Easily implemented as a HashMap<Aaction, Probability>.
pub(crate) trait Policy {
    // required
    fn weight(&self, action: &Self::PAction) -> Probability;
    #[allow(dead_code)]
    fn sample(&self) -> &Self::PAction;

    type PAction: Action;
}
