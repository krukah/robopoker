//! CFR+ regret update strategy.

use super::*;

/// CFR+ regret update strategy.
///
/// Floors regrets at zero after each update, preventing negative regret
/// accumulation. This improves convergence speed significantly for large games.
#[derive(Debug, Clone, Copy, Default)]
pub struct FlooredRegret;

impl RegretSchedule for FlooredRegret {
    fn gain(accumulated: Utility, immediate: Utility, _: usize) -> Utility {
        (accumulated + immediate).max(0.0)
    }
    fn floor() -> Utility {
        0.0
    }
}
