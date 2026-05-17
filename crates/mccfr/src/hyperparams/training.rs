use rbp_core::*;
use rbp_hyperparams::HyperParams;
use std::time::Duration;

/// Training infrastructure parameters.
#[derive(HyperParams, Clone, Copy, Debug, PartialEq)]
pub struct TrainingHyperParams {
    log_interval: Duration,
    flush_interval: Duration,
    mc_exploitability_samples: usize,
    regret_min: Utility,
}

impl TrainingHyperParams {
    pub fn new(
        log_interval: Duration,
        flush_interval: Duration,
        mc_exploitability_samples: usize,
        regret_min: Utility,
    ) -> Self {
        Self {
            log_interval,
            flush_interval,
            mc_exploitability_samples,
            regret_min,
        }
    }

    /// Interval between progress log messages during training.
    pub fn log_interval(&self) -> Duration {
        self.log_interval
    }

    /// Interval between periodic DB flushes during training.
    pub fn flush_interval(&self) -> Duration {
        self.flush_interval
    }

    /// Random deals sampled for Monte Carlo exploitability estimate.
    pub fn mc_exploitability_samples(&self) -> usize {
        self.mc_exploitability_samples
    }

    /// Floor for cumulative regret storage.
    pub fn regret_min(&self) -> Utility {
        self.regret_min
    }
}

impl Default for TrainingHyperParams {
    fn default() -> Self {
        Self {
            log_interval: Duration::from_secs(60),
            flush_interval: Duration::from_secs(30 * 60),
            mc_exploitability_samples: 1024,
            // Floor for cumulative regret storage (prevents unbounded
            // negative growth). Below `PruningHyperParams::threshold` so
            // pruned actions can recover via exploration.
            regret_min: -4e6,
        }
    }
}
