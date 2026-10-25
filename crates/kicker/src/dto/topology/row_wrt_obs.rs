use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RowWrtObs {
    pub obs: Observation,
}

impl RowWrtObs {
    pub fn obs(&self) -> Observation {
        self.obs
    }
}
