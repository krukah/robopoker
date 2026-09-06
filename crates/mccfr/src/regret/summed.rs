//! Vanilla CFR regret accumulation.

use super::*;

/// Adds immediate to accumulated regret with no discounting, so regrets can go
/// arbitrarily negative. The original CFR algorithm.
#[derive(Debug, Clone, Copy, Default)]
pub struct SummedRegret;

impl RegretSchedule for SummedRegret {
    fn accumulate(accumulated: Utility, immediate: Utility, _: usize) -> Utility {
        accumulated + immediate
    }

    fn floor() -> Utility {
        Utility::NEG_INFINITY
    }
}
