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

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRealtimeDiagnostics {
    pub used_dls: bool,
    pub used_blueprint_fallback: bool,
    pub used_legal_fallback: bool,
    pub offtree_detected: bool,
    pub solve_ms: u128,
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRealtimeStrategy {
    pub source: String,
    pub recommended_action: String,
    pub actions: BTreeMap<String, f32>,
    pub diagnostics: ApiRealtimeDiagnostics,
}

// NOTE: impl From<Strategy> for ApiStrategy is in rbp-nlhe
// NOTE: impl From<Decision<Edge>> for ApiDecision is in rbp-nlhe
// NOTE: impl From<tokio_postgres::Row> for ApiSample is in rbp-nlhe or rbp-database
// NOTE: impl From<tokio_postgres::Row> for Decision<Edge> is in rbp-nlhe or rbp-database
