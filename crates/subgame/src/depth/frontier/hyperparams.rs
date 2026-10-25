//! Hyperparameters for depth-limited frontier evaluation.

use pokerkit::Probability;

/// Depth-limited frontier evaluation parameters. The number of continuation
/// strategies each player picks from is the const generic `FRONTIER_LEAVES`,
/// deliberately not a tunable here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrontierHyperParams {
    bias: Probability,
    rollouts: usize,
}

impl FrontierHyperParams {
    /// Zero `rollouts` is normalized to one to keep the divisor in `payoffs()` non-zero.
    pub fn new(bias: Probability, rollouts: usize) -> Self {
        Self {
            bias,
            rollouts: rollouts.max(1),
        }
    }

    /// Multiplier on target action probability when biasing a continuation strategy.
    pub fn bias(&self) -> Probability {
        self.bias
    }

    /// Monte Carlo rollouts per (k, j) continuation pair.
    pub fn rollouts(&self) -> usize {
        self.rollouts
    }
}

impl Default for FrontierHyperParams {
    fn default() -> Self {
        Self {
            bias: 5.0,
            // Monte Carlo rollouts per (k, j) continuation pair for
            // frontier evaluation. More = lower variance, slower solving.
            rollouts: 1 << 4,
        }
    }
}

pokerkit::hyperparams!(FrontierHyperParams);
