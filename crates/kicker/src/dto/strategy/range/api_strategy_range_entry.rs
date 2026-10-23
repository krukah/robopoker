use crate::*;
use deuce::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One concrete pocket's blueprint policy, used to construct a 169-cell
/// strategy range. The client deliberately receives every live combo: after
/// the flop, two hands that share a rank/suitedness cell can differ because
/// their suits interact with the public board differently.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiStrategyRangeEntry {
    pub obs: Observation,
    pub policy: BTreeMap<Edge, f32>,
}

impl ApiStrategyRangeEntry {
    pub fn obs(&self) -> Observation {
        self.obs
    }

    pub fn policy(&self) -> &BTreeMap<Edge, f32> {
        &self.policy
    }
}
