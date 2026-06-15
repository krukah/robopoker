//! Regret update strategies for CFR variants.

mod asymmetric;
mod discounted;
mod floored;
mod linear;
mod summed;

pub use asymmetric::*;
pub use discounted::*;
pub use floored::*;
pub use linear::*;
pub use summed::*;

use pokerkit::*;

/// Trait for regret update strategies in CFR variants.
pub trait RegretSchedule {
    /// Raw regret accumulation before floor.
    fn accumulate(net: Utility, new: Utility, epoch: usize) -> Utility;
    /// Floored regret accumulation (never below floor()).
    fn gain(net: Utility, new: Utility, epoch: usize) -> Utility {
        Self::accumulate(net, new, epoch).max(Self::floor())
    }
    /// Returns the regret floor (minimum regret value).
    fn floor() -> Utility {
        crate::TrainingHyperParams::get().regret_min()
    }
}
