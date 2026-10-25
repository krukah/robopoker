//! Constant weighting scheme.

use super::*;

/// Weights all iterations equally.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConstantWeight;

impl WeightSchedule for ConstantWeight {
    fn accumulate(accumulated: Probability, immediate: Probability, _: usize) -> Probability {
        accumulated + immediate
    }
}
