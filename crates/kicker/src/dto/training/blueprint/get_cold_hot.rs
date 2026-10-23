use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetColdHot {
    #[serde(default = "Request::default_limit")]
    pub limit: i64,
}

impl GetColdHot {
    pub fn limit(&self) -> i64 {
        self.limit
    }
}
