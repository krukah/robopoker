use super::*;
use crate::Case;

/// Evaluation result for one case.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Outcome {
    pub case: Case,
    pub status: Status,
    pub detail: String,
    /// Per-state observations: (label, probability). Single-state kinds
    /// have one entry; pair_diff has 2; monotonic has N.
    pub observed: Vec<(String, Option<f32>)>,
}

impl Outcome {
    pub fn case(&self) -> &Case {
        &self.case
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn observed(&self) -> &[(String, Option<f32>)] {
        &self.observed
    }
}
