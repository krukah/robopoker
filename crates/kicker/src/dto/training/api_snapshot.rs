use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSnapshot {
    pub epoch: i64,
    pub infos: i64,
    pub nodes: i64,
    pub exploit: Option<f32>,
    pub elapsed: i64,
    pub stamped: i64,
}

impl ApiSnapshot {
    pub fn epoch(&self) -> i64 {
        self.epoch
    }

    pub fn infos(&self) -> i64 {
        self.infos
    }

    pub fn nodes(&self) -> i64 {
        self.nodes
    }

    pub fn exploit(&self) -> Option<f32> {
        self.exploit
    }

    pub fn elapsed(&self) -> i64 {
        self.elapsed
    }

    pub fn stamped(&self) -> i64 {
        self.stamped
    }
}
