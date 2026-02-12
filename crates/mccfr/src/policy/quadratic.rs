//! Quadratic weighting scheme.

use super::*;

/// Quadratic weighting scheme.
///
/// Later iterations are weighted by the square of their iteration number.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuadraticWeight;

impl PolicySchedule for QuadraticWeight {
    fn learn(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability {
        let t = epoch as Probability;
        (accumulated + immediate * t * t).max(POLICY_MIN)
    }
    fn discount(epoch: usize) -> Probability {
        let t = epoch as f32;
        (t / (t + 1.0)).powf(2.)
    }
}
