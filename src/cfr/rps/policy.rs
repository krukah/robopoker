use super::action::RpsEdge;
use crate::cfr::training::learning::policy::Policy;
use crate::cfr::training::Probability;
use std::collections::HashMap;

pub(crate) struct RpsPolicy {
    weights: HashMap<RpsEdge, Probability>,
}

impl RpsPolicy {
    pub fn new() -> Self {
        Self {
            weights: HashMap::new(),
        }
    }
}

impl Policy for RpsPolicy {
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

    type PAction = RpsEdge;
}
