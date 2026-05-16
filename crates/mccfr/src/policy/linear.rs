//! Linear weighting scheme.

use super::*;

/// Linear weighting scheme.
///
/// Later iterations are weighted proportionally to their iteration number.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearWeight;

impl WeightSchedule for LinearWeight {
    fn accumulate(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability {
        let t = epoch as Probability;
        accumulated + immediate * t
    }
}
