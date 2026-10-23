use super::*;
use serde::{Deserialize, Serialize};

/// Blueprint policy for every live private holding at one decision point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiStrategyRange {
    pub entries: Vec<ApiStrategyRangeEntry>,
}

impl ApiStrategyRange {
    pub fn entries(&self) -> &[ApiStrategyRangeEntry] {
        &self.entries
    }
}
