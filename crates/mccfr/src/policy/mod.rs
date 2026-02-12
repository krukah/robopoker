//! Policy weighting and strategy distributions.

mod constant;
mod distribution;
mod exponential;
mod linear;
mod quadratic;

pub use constant::*;
pub use distribution::*;
pub use exponential::*;
pub use linear::*;
pub use quadratic::*;

use rbp_core::*;

/// Trait for strategy weighting schemes in CFR.
pub trait PolicySchedule {
    /// Updates policy value with the new policy weight.
    fn learn(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability;
    /// Returns the discount factor for accumulated policies.
    fn discount(epoch: usize) -> Probability {
        let _ = epoch;
        1.0
    }
}
