use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ObsHist {
    pub obs: Observation,
}

impl ObsHist {
    pub fn obs(&self) -> Observation {
        self.obs
    }
}
