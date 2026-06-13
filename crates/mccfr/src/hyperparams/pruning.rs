use rbp_core::*;
use rbp_hyperparams::HyperParams;

/// Probabilistic pruning parameters (see PluribusSampling).
#[derive(HyperParams, Clone, Copy, Debug, PartialEq)]
pub struct PruningHyperParams {
    threshold: Utility,
    explore: Probability,
    warmup: usize,
}

impl PruningHyperParams {
    pub fn new(threshold: Utility, explore: Probability, warmup: usize) -> Self {
        Self {
            threshold,
            explore,
            warmup,
        }
    }

    /// Regret floor below which actions are candidates for pruning.
    pub fn threshold(&self) -> Utility {
        self.threshold
    }

    /// Probability of sampling pruned actions anyway.
    pub fn explore(&self) -> Probability {
        self.explore
    }

    /// Warm-up epochs before pruning activates.
    pub fn warmup(&self) -> usize {
        self.warmup
    }
}

impl Default for PruningHyperParams {
    fn default() -> Self {
        Self {
            // Actions with regret below this are candidates for pruning
            // (-300k ≈ 3× max pot). Above the regret floor so floored
            // actions can recover via exploration.
            threshold: -3e5,
            // Probability of sampling pruned actions anyway.
            explore: 0.05,
            // Warm-up epochs before pruning activates. One epoch = one
            // CFR step = `batch_size` trees (128 for NLHE Nlhe, see
            // `mccfr!()` invocation in `crates/nlhe/src/solver.rs`).
            // 16384 epochs × 128 trees = ~2.1M trees. On an r6i.8xlarge
            // at ~300k infosets/sec this completes in roughly 5 minutes
            // — pruning is effectively active for the entire run.
            warmup: 16_384,
        }
    }
}
