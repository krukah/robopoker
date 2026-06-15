//! Exponential weighting scheme.

use super::*;

/// Exponential weighting scheme.
///
/// Uses exponential weighting with configurable decay rate.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExponentialWeight;

impl ExponentialWeight {
    const DECAY: f32 = 0.9999;
}

impl WeightSchedule for ExponentialWeight {
    fn accumulate(accumulated: Probability, immediate: Probability, _: usize) -> Probability {
        accumulated * Self::DECAY + immediate
    }
}
