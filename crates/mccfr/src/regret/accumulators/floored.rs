//! CFR+ regret update strategy.

use super::*;

/// Floors regret at zero after each update, forbidding negative accumulation.
/// Converges significantly faster on large games.
#[derive(Debug, Clone, Copy, Default)]
pub struct FlooredRegret;

impl RegretSchedule for FlooredRegret {
    fn accumulate(accumulated: Utility, immediate: Utility, _: usize) -> Utility {
        accumulated + immediate
    }

    fn floor() -> Utility {
        0.0
    }
}
