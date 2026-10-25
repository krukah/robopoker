use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReplaceObs {
    pub obs: Observation,
}

impl ReplaceObs {
    pub fn obs(&self) -> Observation {
        self.obs
    }
}
