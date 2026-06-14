//! Hyperparameters for depth-limited frontier evaluation.

use pokerkit::Probability;
use solus::HyperParams;

/// Depth-limited frontier evaluation parameters.
///
/// At depth-limited frontiers, each player picks from a fixed number
/// of biased continuation strategies (the count is the const generic
/// `FRONTIER_LEAVES`, not part of these tunables). The solver learns
/// the minimax mix over the resulting payoff matrix.
#[derive(HyperParams, Clone, Copy, Debug, PartialEq)]
pub struct FrontierHyperParams {
    bias: Probability,
    rollouts: usize,
}

impl FrontierHyperParams {
    pub fn new(bias: Probability, rollouts: usize) -> Self {
        Self { bias, rollouts }
    }

    /// Multiplier on target action probability when biasing a continuation strategy.
    pub fn bias(&self) -> Probability {
        self.bias
    }

    /// Monte Carlo rollouts per (k, j) continuation pair. Zero is normalized
    /// to one to keep the divisor in `payoffs()` non-zero.
    pub fn rollouts(&self) -> usize {
        self.rollouts.max(1)
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
