//! Vanilla CFR regret accumulation.

use super::*;

/// Vanilla CFR regret accumulation.
///
/// Simply adds immediate regret to accumulated regret with no discounting.
/// Regrets can go arbitrarily negative. This is the original CFR algorithm.
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
