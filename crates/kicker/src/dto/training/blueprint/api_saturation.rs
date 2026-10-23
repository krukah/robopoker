use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSaturation {
    pub max_weight: f32,
    pub max_regret: f32,
    pub max_payoff: f32,
    pub max_visits: i32,
    pub precision_f32: f32,
    pub weight_pct: f32,
    pub regret_pct: f32,
}

impl ApiSaturation {
    pub fn max_weight(&self) -> f32 {
        self.max_weight
    }

    pub fn max_regret(&self) -> f32 {
        self.max_regret
    }

    pub fn max_payoff(&self) -> f32 {
        self.max_payoff
    }

    pub fn max_visits(&self) -> i32 {
        self.max_visits
    }

    pub fn precision_f32(&self) -> f32 {
        self.precision_f32
    }

    pub fn weight_pct(&self) -> f32 {
        self.weight_pct
    }

    pub fn regret_pct(&self) -> f32 {
        self.regret_pct
    }
}
