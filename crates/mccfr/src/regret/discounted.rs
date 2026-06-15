//! Discounted CFR regret schedule (DCFR, α=1.5, β=0.5).

use super::*;

/// Discounted CFR regret schedule.
///
/// Applies asymmetric discounting to positive and negative regrets,
/// following Brown & Sandholm, "Solving Imperfect-Information Games via
/// Discounted Regret Minimization" (AAAI 2019).
///
/// - **Positive regrets**: Discounted by t^α/(t^α+1), α=1.5
/// - **Negative regrets**: Discounted by t^β/(t^β+1), β=0.5
///
/// Not the variant used in Pluribus itself — Pluribus used Linear CFR
/// (i.e. [`LinearRegret`]). These are the general-purpose DCFR defaults
/// recommended in the AAAI paper.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiscountedRegret;

impl DiscountedRegret {
    const ALPHA: f32 = 1.5;
    const BETA: f32 = 0.5;
    const PERIOD: usize = 1;
}

impl RegretSchedule for DiscountedRegret {
    fn accumulate(accumulated: Utility, immediate: Utility, epoch: usize) -> Utility {
        let t = epoch as f32;
        let p = Self::PERIOD as f32;
        if !epoch.is_multiple_of(Self::PERIOD) {
            accumulated + immediate
        } else if accumulated > 0.0 {
            let x = (t / p).powf(Self::ALPHA);
            let discount = x / (x + 1.0);
            accumulated * discount + immediate
        } else if accumulated < 0.0 {
            let x = (t / p).powf(Self::BETA);
            let discount = x / (x + 1.0);
            accumulated * discount + immediate
        } else {
            let x = t / p;
            let discount = x / (x + 1.0);
            accumulated * discount + immediate
        }
    }
}
