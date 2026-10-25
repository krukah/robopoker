use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStreetStats {
    pub street: String,
    pub infosets: i64,
    pub edges: i64,
    pub avg_regret: f32,
    pub avg_weight: f32,
    pub avg_payoff: f32,
    pub avg_visits: f32,
}

impl ApiStreetStats {
    pub fn street(&self) -> &str {
        &self.street
    }

    pub fn infosets(&self) -> i64 {
        self.infosets
    }

    pub fn edges(&self) -> i64 {
        self.edges
    }

    pub fn avg_regret(&self) -> f32 {
        self.avg_regret
    }

    pub fn avg_weight(&self) -> f32 {
        self.avg_weight
    }

    pub fn avg_payoff(&self) -> f32 {
        self.avg_payoff
    }

    pub fn avg_visits(&self) -> f32 {
        self.avg_visits
    }
}
