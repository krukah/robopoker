//! Constant weighting scheme.

use super::*;

/// Constant weighting scheme.
///
/// All iterations are weighted equally.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConstantWeight;

impl WeightSchedule for ConstantWeight {
    fn accumulate(accumulated: Probability, immediate: Probability, _: usize) -> Probability {
        accumulated + immediate
    }
}
