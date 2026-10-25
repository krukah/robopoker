use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDecision {
    pub edge: Edge,
    pub mass: f32,
}

impl ApiDecision {
    pub fn edge(&self) -> Edge {
        self.edge
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }
}
