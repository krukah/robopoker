use crate::*;
use kicker::*;
use pokerkit::Config;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSample {
    pub obs: Observation,
    pub abs: Abstraction,
    pub equity: f32,
    pub density: f32,
    pub distance: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDecision {
    pub edge: Edge,
    pub mass: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStrategy {
    pub history: Path,
    pub present: Abstraction,
    pub choices: Path,
    pub spr: u8,
    pub accumulated: BTreeMap<Edge, f32>,
    pub visits: BTreeMap<Edge, u32>,
    pub payoff: f32,
}

/// Server-side projection of [`Config`]'s `(depth, world)` axes to the
/// route that produced an [`ApiSolved`]. Dirac lives off-wire — it's a
/// client-side post-process. The four routes are the four corners of
/// the (depth, world) sub-cube:
///
/// | depth | world | Kind        | Route                  |
/// | ----- | ----- | ----------- | ---------------------- |
/// | false | false | `Blueprint` | `/strategy/policy`     |
/// | true  | false | `Depth`     | `/strategy/depth`      |
/// | false | true  | `World`     | `/strategy/world`      |
/// | true  | true  | `Full`      | `/strategy/full`       |
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Blueprint,
    Depth,
    World,
    Full,
}

impl Kind {
    /// Wire-format label, also used as the route segment and the FE
    /// provenance strip. Single source of truth so client and server
    /// agree.
    pub fn label(self) -> &'static str {
        match self {
            Self::Blueprint => "blueprint",
            Self::Depth => "depth",
            Self::World => "world",
            Self::Full => "full",
        }
    }
    /// Project a cube [`Config`] onto its route corner. `dirac` is
    /// post-processing on top of any of these — it doesn't pick a
    /// kind, it transforms the resulting distribution client-side.
    pub fn from_config(config: Config) -> Self {
        match (config.depth, config.world) {
            (false, false) => Self::Blueprint,
            (true, false) => Self::Depth,
            (false, true) => Self::World,
            (true, true) => Self::Full,
        }
    }
}

/// Wraps an [`ApiStrategy`] with provenance — which source produced
/// it, how many MCCFR iterations the solver ran (1 for blueprint),
/// and how long the request took on the server. Lets the client
/// surface variance across re-runs of stochastic solvers without
/// changing the policy-rendering path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSolved {
    pub kind: Kind,
    pub policy: ApiStrategy,
    pub iterations: u32,
    pub elapsed_ms: u32,
}

impl ApiSolved {
    /// Constructor for the cheap blueprint path — DB lookup, 1 iter,
    /// `elapsed` from the request handler's timer.
    pub fn blueprint(policy: ApiStrategy, elapsed: Duration) -> Self {
        Self {
            kind: Kind::Blueprint,
            policy,
            iterations: 1,
            elapsed_ms: elapsed.as_millis() as u32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiOpponentRange {
    /// Hole-card-level posterior, normalized to sum to 1. Each entry is
    /// the probability that villain holds that exact pocket given the
    /// observed action history under the blueprint.
    pub entries: Vec<ApiRangeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRangeEntry {
    pub obs: Observation,
    pub weight: f32,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStatus {
    pub epoch: i64,
    pub infosets: i64,
    pub exploit: Option<f32>,
    pub stamped: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSnapshot {
    pub epoch: i64,
    pub infos: i64,
    pub nodes: i64,
    pub exploit: Option<f32>,
    pub elapsed: i64,
    pub stamped: i64,
}

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
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiColdInfoset {
    pub past: i64,
    pub present: i16,
    pub choices: i64,
    pub visits: i32,
    pub edges: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiHotInfoset {
    pub past: i64,
    pub present: i16,
    pub choices: i64,
    pub max_regret: f32,
    pub edges: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConvergence {
    pub epoch: i64,
    pub exploit: f32,
    pub delta: f32,
    pub stamped: i64,
}
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRecap {
    pub pot: i16,
    pub board: String,
    pub dealer: usize,
    pub players: Vec<ApiPlayer>,
    pub actions: Vec<ApiPlay>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiPlayer {
    pub seat: usize,
    pub stack: i16,
    pub hole: Option<String>,
    pub won: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiPlay {
    pub seq: i16,
    pub action: String,
    pub street: String,
}
