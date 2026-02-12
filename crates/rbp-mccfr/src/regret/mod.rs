//! Regret update strategies for CFR variants.

mod discounted;
mod floored;
mod linear;
mod pluribus;
mod summed;

pub use discounted::*;
pub use floored::*;
pub use linear::*;
pub use pluribus::*;
pub use summed::*;

use rbp_core::*;

/// Trait for regret update strategies in CFR variants.
pub trait RegretSchedule {
    /// Updates regret value with the new regret gain.
    fn gain(net: Utility, new: Utility, epoch: usize) -> Utility;
    /// Returns the regret floor (minimum regret value).
    fn floor() -> Utility {
        REGRET_MIN
    }
}
