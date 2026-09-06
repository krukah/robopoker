//! Quadratic weighting scheme.

use super::*;

/// Weights each iteration by the square of its iteration number.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuadraticWeight;

impl WeightSchedule for QuadraticWeight {
    fn accumulate(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability {
        let t = epoch as Probability;
        accumulated + immediate * t * t
    }
}
