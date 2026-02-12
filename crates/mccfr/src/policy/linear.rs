//! Linear weighting scheme.

use super::*;

/// Linear weighting scheme.
///
/// Later iterations are weighted proportionally to their iteration number.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearWeight;

impl LinearWeight {
    const GAMMA: f32 = 1.5;
}

impl PolicySchedule for LinearWeight {
    fn learn(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability {
        let t = epoch as Probability;
        (accumulated + immediate * t).max(POLICY_MIN)
    }
    fn discount(epoch: usize) -> Probability {
        let t = epoch as f32;
        (t / (t + 1.0)).powf(Self::GAMMA)
    }
}
