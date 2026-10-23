use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiHotInfoset {
    pub past: Subgame,
    pub present: Abstraction,
    pub choices: Path,
    pub max_regret: f32,
    pub edges: i64,
}

impl ApiHotInfoset {
    pub fn past(&self) -> Subgame {
        self.past
    }

    pub fn present(&self) -> Abstraction {
        self.present
    }

    pub fn choices(&self) -> Path {
        self.choices
    }

    pub fn max_regret(&self) -> f32 {
        self.max_regret
    }

    pub fn edges(&self) -> i64 {
        self.edges
    }
}
