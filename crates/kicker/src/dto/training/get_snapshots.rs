use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetSnapshots {
    #[serde(default = "Request::default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

impl GetSnapshots {
    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }
}
