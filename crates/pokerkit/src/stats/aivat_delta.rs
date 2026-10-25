use super::*;
use serde::{Deserialize, Serialize};

/// AIVAT-adjusted aggregate: lean wire type with no embedded Summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AivatDelta {
    pub series: Vec<Utility>,
    pub won: Utility,
    pub stderr: Utility,
    pub reduction: f32,
    pub pvalue: f32,
}

impl AivatDelta {
    pub fn series(&self) -> &[Utility] {
        &self.series
    }

    pub fn won(&self) -> Utility {
        self.won
    }

    pub fn stderr(&self) -> Utility {
        self.stderr
    }

    pub fn reduction(&self) -> f32 {
        self.reduction
    }

    pub fn pvalue(&self) -> f32 {
        self.pvalue
    }
}
