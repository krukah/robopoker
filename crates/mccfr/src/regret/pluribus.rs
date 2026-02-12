//! Pluribus regret schedule (hybrid linear/discounted).

use super::*;

/// Pluribus regret schedule (hybrid linear/discounted).
///
/// - **Positive regrets**: No discounting — accumulate directly
/// - **Negative regrets**: Discounted by t/(t+1) — decay toward zero
#[derive(Debug, Clone, Copy, Default)]
pub struct PluribusRegret;

impl RegretSchedule for PluribusRegret {
    fn gain(accumulated: Utility, immediate: Utility, epoch: usize) -> Utility {
        let t = epoch as f32;
        if accumulated > 0.0 {
            (accumulated + immediate).max(REGRET_MIN)
        } else {
            let discount = t / (t + 1.0);
            (accumulated * discount + immediate).max(REGRET_MIN)
        }
    }
}
