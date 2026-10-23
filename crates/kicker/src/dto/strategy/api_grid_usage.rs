use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiGridUsage {
    pub street: String,
    pub edge: String,
    /// Per-decision arithmetic mean of strategy probability — each
    /// (past, present, choices) tuple weighted equally regardless
    /// of visit count. Overweights the long tail of rare decisions.
    pub avg_freq: f32,
    /// Visit-weighted strategy probability — approximates the
    /// frequency of this action in real games. `Σ weight(edge) /
    /// Σ total_weight` across decisions where the edge was available.
    pub weighted_freq: f32,
    pub n_decisions_with_edge: i64,
    pub n_dominant: i64,
}

impl ApiGridUsage {
    pub fn street(&self) -> &str {
        &self.street
    }

    pub fn edge(&self) -> &str {
        &self.edge
    }

    pub fn avg_freq(&self) -> f32 {
        self.avg_freq
    }

    pub fn weighted_freq(&self) -> f32 {
        self.weighted_freq
    }

    pub fn n_decisions_with_edge(&self) -> i64 {
        self.n_decisions_with_edge
    }

    pub fn n_dominant(&self) -> i64 {
        self.n_dominant
    }
}
