//! Discounted CFR (DCFR) regret update strategy.

use super::*;

/// Discounted CFR (DCFR) regret update strategy.
///
/// Applies asymmetric discounting to positive and negative regrets,
/// with stronger discounting for negative regrets.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiscountedRegret;

impl DiscountedRegret {
    const ALPHA: f32 = 1.5;
    const BETA: f32 = 0.5;
    const PERIOD: usize = 1;
}

impl RegretSchedule for DiscountedRegret {
    fn gain(accumulated: Utility, immediate: Utility, epoch: usize) -> Utility {
        let t = epoch as f32;
        let p = Self::PERIOD as f32;
        if (epoch % Self::PERIOD) != 0 {
            accumulated + immediate
        } else if accumulated > 0.0 {
            let x = (t / p).powf(Self::ALPHA);
            let discount = x / (x + 1.0);
            (accumulated * discount + immediate).max(REGRET_MIN)
        } else if accumulated < 0.0 {
            let x = (t / p).powf(Self::BETA);
            let discount = x / (x + 1.0);
            (accumulated * discount + immediate).max(REGRET_MIN)
        } else {
            let x = t / p;
            let discount = x / (x + 1.0);
            (accumulated * discount + immediate).max(REGRET_MIN)
        }
    }
}
