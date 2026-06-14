//! Asymmetric regret schedule (undiscounted positive, linear-decayed negative).

use super::*;

/// Asymmetric regret schedule (undiscounted positive, linear-decayed negative).
///
/// - **Positive regrets**: No discounting — accumulate directly
/// - **Negative regrets**: Discounted by t/(t+1) — decay toward zero
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
