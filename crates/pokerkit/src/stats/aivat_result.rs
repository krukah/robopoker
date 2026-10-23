use super::*;
use serde::{Deserialize, Serialize};

/// AIVAT-adjusted result for a single hand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AivatResult {
    pub raw: Chips,
    pub adjusted: Utility,
    pub corrections: Utility,
    pub hero_corrections: Utility,
    pub villain_corrections: Utility,
    pub chance_corrections: Utility,
}

impl AivatResult {
    pub fn raw(&self) -> Chips {
        self.raw
    }

    pub fn adjusted(&self) -> Utility {
        self.adjusted
    }

    pub fn corrections(&self) -> Utility {
        self.corrections
    }

    pub fn hero_corrections(&self) -> Utility {
        self.hero_corrections
    }

    pub fn villain_corrections(&self) -> Utility {
        self.villain_corrections
    }

    pub fn chance_corrections(&self) -> Utility {
        self.chance_corrections
    }
}
