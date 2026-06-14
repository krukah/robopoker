//! Hyperparameters for safe + depth-limited subgame solving.

use horizon::FrontierHyperParams;
use solus::HyperParams;

/// Subgame solving parameters.
///
/// Controls per-decision real-time refinement of the blueprint strategy.
/// Composes [`FrontierHyperParams`] for the depth-limited leaf evaluation.
#[derive(HyperParams, Clone, Copy, Debug, PartialEq)]
pub struct SubgameHyperParams {
    timeout_ms: u64,
    visit_threshold: u32,
    frontier: FrontierHyperParams,
}

impl SubgameHyperParams {
    pub fn new(timeout_ms: u64, visit_threshold: u32, frontier: FrontierHyperParams) -> Self {
        Self {
            timeout_ms,
            visit_threshold,
            frontier,
        }
    }

    /// Time budget (ms) for real-time subgame solving per decision.
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// Per-edge visit count at which subgame's policy gets 50% weight.
    pub fn visit_threshold(&self) -> u32 {
        self.visit_threshold
    }

    /// Depth-limited frontier evaluation parameters.
    pub fn frontier(&self) -> &FrontierHyperParams {
        &self.frontier
    }
}

impl Default for SubgameHyperParams {
    fn default() -> Self {
        Self {
            // Time budget (ms) for real-time subgame solving per decision.
            timeout_ms: 5000,
            // Visit-count threshold at which subgame's policy gets 50%
            // weight in the refined extraction. Per-action weight:
            // `w(a) = visits(a) / (visits(a) + V)`. Sits near observed
            // p50 visits → ~55-65% subgame / ~35-45% blueprint blend.
            visit_threshold: 1 << 18,
            frontier: FrontierHyperParams::default(),
        }
    }
}
