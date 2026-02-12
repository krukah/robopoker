//! Linear discounting for CFR-D.

use super::*;

/// Linear discounting for CFR-D.
///
/// Applies linear time-weighted discounting where older iterations
/// are weighted proportionally less. Uses discount factor t/(t+1) at each epoch.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinearRegret;

impl RegretSchedule for LinearRegret {
    fn gain(accumulated: Utility, immediate: Utility, epoch: usize) -> Utility {
        let t = epoch as f32;
        let discount = t / (t + 1.0);
        (accumulated * discount + immediate).max(REGRET_MIN)
    }
}
