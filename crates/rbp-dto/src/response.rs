use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSample {
    pub obs: String,
    pub abs: String,
    pub equity: f32,
    pub density: f32,
    pub distance: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDecision {
    pub edge: String,
    pub mass: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStrategy {
    pub history: i64,
    pub present: i16,
    pub choices: i64,
    pub accumulated: BTreeMap<String, f32>,
    pub counts: BTreeMap<String, u32>,
}

// NOTE: impl From<Strategy> for ApiStrategy is in rbp-nlhe
// NOTE: impl From<Decision<Edge>> for ApiDecision is in rbp-nlhe
// NOTE: impl From<tokio_postgres::Row> for ApiSample is in rbp-nlhe or rbp-database
// NOTE: impl From<tokio_postgres::Row> for Decision<Edge> is in rbp-nlhe or rbp-database
