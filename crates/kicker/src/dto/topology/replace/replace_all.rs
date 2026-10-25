use crate::*;
use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReplaceAll {
    pub wrt: Abstraction,
    pub neighbors: Vec<Observation>,
}

impl ReplaceAll {
    pub fn wrt(&self) -> Abstraction {
        self.wrt
    }

    pub fn neighbors(&self) -> &[Observation] {
        &self.neighbors
    }
}
