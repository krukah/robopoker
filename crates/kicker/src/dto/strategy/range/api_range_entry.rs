use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiRangeEntry {
    pub obs: Observation,
    pub weight: f32,
}

impl ApiRangeEntry {
    pub fn obs(&self) -> Observation {
        self.obs
    }

    pub fn weight(&self) -> f32 {
        self.weight
    }
}
