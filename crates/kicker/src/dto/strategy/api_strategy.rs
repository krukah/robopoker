use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStrategy {
    pub history: Subgame,
    pub present: Abstraction,
    pub choices: Path,
    pub field: Field,
    pub spr: u8,
    pub accumulated: BTreeMap<Edge, f32>,
    pub visits: BTreeMap<Edge, u32>,
    pub payoff: f32,
}

impl ApiStrategy {
    pub fn history(&self) -> Subgame {
        self.history
    }

    pub fn present(&self) -> Abstraction {
        self.present
    }

    pub fn choices(&self) -> Path {
        self.choices
    }

    pub fn field(&self) -> Field {
        self.field
    }

    pub fn spr(&self) -> u8 {
        self.spr
    }

    pub fn accumulated(&self) -> &BTreeMap<Edge, f32> {
        &self.accumulated
    }

    pub fn visits(&self) -> &BTreeMap<Edge, u32> {
        &self.visits
    }

    pub fn payoff(&self) -> f32 {
        self.payoff
    }
}
