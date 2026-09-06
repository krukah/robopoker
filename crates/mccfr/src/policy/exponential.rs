//! Exponential weighting scheme.

use super::*;

/// Discounts prior weight by a fixed decay rate each epoch.
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
