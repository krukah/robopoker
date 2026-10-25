use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConvergence {
    pub epoch: i64,
    pub exploit: f32,
    pub delta: f32,
    pub stamped: i64,
}

impl ApiConvergence {
    pub fn epoch(&self) -> i64 {
        self.epoch
    }

    pub fn exploit(&self) -> f32 {
        self.exploit
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn stamped(&self) -> i64 {
        self.stamped
    }
}
