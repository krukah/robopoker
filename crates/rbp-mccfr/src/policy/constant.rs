//! Constant weighting scheme.

use super::*;

/// Constant weighting scheme.
///
/// All iterations are weighted equally.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConstantWeight;

impl PolicySchedule for ConstantWeight {
    fn learn(accumulated: Probability, immediate: Probability, _: usize) -> Probability {
        (accumulated + immediate).max(POLICY_MIN)
    }
}
