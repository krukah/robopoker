use super::action::RpsAction;
use crate::cfr::traits::training::policy::Policy;
use crate::cfr::traits::Probability;
use std::collections::HashMap;

impl Policy for HashMap<RpsAction, Probability> {
    fn weight(&self, action: &Self::PAction) -> Probability {
        *self
            .get(action)
            .expect("weight initialized across action set")
    }
    fn sample(&self) -> &Self::PAction {
        self.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0
    }
    type PAction = RpsAction;
}
