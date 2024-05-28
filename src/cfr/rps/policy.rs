use super::action::RPSEdge;
use crate::cfr::training::learning::policy::Policy;
use crate::cfr::training::Probability;
use std::collections::HashMap;

pub(crate) struct RPSPolicy {
    weights: HashMap<RPSEdge, Probability>,
}

impl RPSPolicy {
    pub fn new() -> Self {
        Self {
            weights: HashMap::new(),
        }
    }
}

impl Policy for RPSPolicy {
    fn weight(&self, action: &Self::PAction) -> Probability {
        *self
            .weights
            .get(action)
            .expect("weights initialized across action set")
    }
    fn sample(&self) -> &Self::PAction {
        self.weights
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0
    }

    type PAction = RPSEdge;
}
