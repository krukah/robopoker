use crate::*;
use rbp_cards::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SetStreets {
    pub street: Street,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceObs {
    pub obs: Observation,
}

#[derive(Serialize, Deserialize)]
pub struct RowWrtObs {
    pub obs: Observation,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceAbs {
    pub wrt: Abstraction,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceRow {
    pub wrt: Abstraction,
    pub obs: Observation,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceOne {
    pub wrt: Abstraction,
    pub abs: Abstraction,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceAll {
    pub wrt: Abstraction,
    pub neighbors: Vec<Observation>,
}

#[derive(Serialize, Deserialize)]
pub struct ObsHist {
    pub obs: Observation,
}

#[derive(Serialize, Deserialize)]
pub struct AbsHist {
    pub abs: Abstraction,
}

#[derive(Serialize, Deserialize)]
pub struct GetPolicy {
    pub turn: Turn,
    pub seen: Observation,
    pub past: Vec<Action>,
}

impl From<&Witness> for GetPolicy {
    fn from(recall: &Witness) -> Self {
        Self {
            turn: recall.turn(),
            seen: recall.seen(),
            past: recall
                .actions()
                .iter()
                .filter(|a| a.is_choice())
                .copied()
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetDistance {
    pub a: String,
    pub b: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetSnapshots {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn default_limit() -> i64 {
    100
}

#[derive(Serialize, Deserialize)]
pub struct GetColdHot {
    #[serde(default = "default_limit")]
    pub limit: i64,
}
#[derive(Serialize, Deserialize)]
pub struct GetSummary {
    pub user: uuid::Uuid,
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub against: Option<uuid::Uuid>,
    #[serde(default)]
    pub stakes: Option<i16>,
    #[serde(default)]
    pub hero_human: bool,
    #[serde(default)]
    pub against_human: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetHandRecap {
    pub id: uuid::Uuid,
}
