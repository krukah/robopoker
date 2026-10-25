use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiBlueprintStats {
    pub infosets: i64,
    pub edges: i64,
    pub avg_regret: f32,
    pub max_regret: f32,
    pub min_regret: f32,
    pub avg_weight: f32,
    pub max_weight: f32,
    pub avg_payoff: f32,
    pub max_payoff: f32,
    pub min_payoff: f32,
    pub avg_visits: f32,
    pub max_visits: i32,
    pub min_visits: i32,
}

impl ApiBlueprintStats {
    pub fn infosets(&self) -> i64 {
        self.infosets
    }

    pub fn edges(&self) -> i64 {
        self.edges
    }

    pub fn avg_regret(&self) -> f32 {
        self.avg_regret
    }

    pub fn max_regret(&self) -> f32 {
        self.max_regret
    }

    pub fn min_regret(&self) -> f32 {
        self.min_regret
    }

    pub fn avg_weight(&self) -> f32 {
        self.avg_weight
    }

    pub fn max_weight(&self) -> f32 {
        self.max_weight
    }

    pub fn avg_payoff(&self) -> f32 {
        self.avg_payoff
    }

    pub fn max_payoff(&self) -> f32 {
        self.max_payoff
    }

    pub fn min_payoff(&self) -> f32 {
        self.min_payoff
    }

    pub fn avg_visits(&self) -> f32 {
        self.avg_visits
    }

    pub fn max_visits(&self) -> i32 {
        self.max_visits
    }

    pub fn min_visits(&self) -> i32 {
        self.min_visits
    }
}
