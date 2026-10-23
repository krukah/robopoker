use crate::*;
use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReplaceRow {
    pub wrt: Abstraction,
    pub obs: Observation,
}

impl ReplaceRow {
    pub fn wrt(&self) -> Abstraction {
        self.wrt
    }

    pub fn obs(&self) -> Observation {
        self.obs
    }
}
