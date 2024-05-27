use super::{action::Action, Probability};

/// A policy (P: node -> prob) is a distribution over A(Ii). Easily implemented as a HashMap<Aaction, Probability>.
pub(crate) trait Policy {
    // required
    fn weights(&self, action: &Self::PAction) -> Probability;
    fn sample(&self) -> &Self::PAction;

    type PAction: Action;
}
