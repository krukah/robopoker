use super::action::RPSEdge;
use crate::cfr::training::{policy::Policy, Probability};
use std::collections::HashMap;

/// tabular Action > Probability
pub(crate) struct RPSPolicy {
    weights: HashMap<RPSEdge, Probability>,
}

impl RPSPolicy {
    pub fn new(weights: HashMap<RPSEdge, Probability>) -> Self {
        Self { weights }
    }
}
impl Policy for RPSPolicy {
    type PAction = RPSEdge;
    fn weights(&self, action: &Self::PAction) -> Probability {
        *self
            .weights
            .get(action)
            .expect("no weight stored for action, is this a new tree?")
    }
    fn sample(&self) -> &Self::PAction {
        self.weights
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0
    }
}
