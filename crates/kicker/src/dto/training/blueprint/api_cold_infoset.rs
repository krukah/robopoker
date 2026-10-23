use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiColdInfoset {
    pub past: Subgame,
    pub present: Abstraction,
    pub choices: Path,
    pub visits: i32,
    pub edges: i64,
}

impl ApiColdInfoset {
    pub fn past(&self) -> Subgame {
        self.past
    }

    pub fn present(&self) -> Abstraction {
        self.present
    }

    pub fn choices(&self) -> Path {
        self.choices
    }

    pub fn visits(&self) -> i32 {
        self.visits
    }

    pub fn edges(&self) -> i64 {
        self.edges
    }
}
