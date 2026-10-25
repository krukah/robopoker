use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStatus {
    pub epoch: i64,
    pub infosets: i64,
    pub exploit: Option<f32>,
    pub stamped: Option<i64>,
}

impl ApiStatus {
    pub fn epoch(&self) -> i64 {
        self.epoch
    }

    pub fn infosets(&self) -> i64 {
        self.infosets
    }

    pub fn exploit(&self) -> Option<f32> {
        self.exploit
    }

    pub fn stamped(&self) -> Option<i64> {
        self.stamped
    }
}
