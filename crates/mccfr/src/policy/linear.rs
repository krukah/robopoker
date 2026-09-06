//! Linear weighting scheme.

use super::*;

/// Weights each iteration proportionally to its iteration number.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearWeight;

impl WeightSchedule for LinearWeight {
    fn accumulate(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability {
        let t = epoch as Probability;
        accumulated + immediate * t
    }
}
