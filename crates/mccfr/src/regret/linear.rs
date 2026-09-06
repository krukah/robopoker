//! Linear discounting for CFR-D.

use super::*;

/// Time-weighted discounting by `t/(t+1)` each epoch, so older iterations count
/// proportionally less.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearRegret;

impl RegretSchedule for LinearRegret {
    fn accumulate(accumulated: Utility, immediate: Utility, epoch: usize) -> Utility {
        let t = epoch as f32;
        let discount = t / (t + 1.0);
        accumulated * discount + immediate
    }
}
