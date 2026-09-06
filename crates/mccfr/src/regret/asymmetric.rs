//! Asymmetric regret schedule (undiscounted positive, linear-decayed negative).

use super::*;

/// Positive regrets accumulate undiscounted; negative regrets decay toward zero
/// by `t/(t+1)`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AsymmetricRegret;

impl RegretSchedule for AsymmetricRegret {
    fn accumulate(accumulated: Utility, immediate: Utility, epoch: usize) -> Utility {
        let t = epoch as f32;
        if accumulated > 0.0 {
            accumulated + immediate
        } else {
            let discount = t / (t + 1.0);
            accumulated * discount + immediate
        }
    }
}
