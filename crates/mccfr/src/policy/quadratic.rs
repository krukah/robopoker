//! Quadratic weighting scheme.

use super::*;

/// Quadratic weighting scheme.
///
/// Later iterations are weighted by the square of their iteration number.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuadraticWeight;

impl WeightSchedule for QuadraticWeight {
    fn accumulate(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability {
        let t = epoch as Probability;
        accumulated + immediate * t * t
    }
}
