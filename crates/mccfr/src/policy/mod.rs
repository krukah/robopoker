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

use pokerkit::*;

/// How each iteration's strategy is weighted into the running average.
pub trait WeightSchedule {
    /// Raw accumulation before epsilon floor.
    fn accumulate(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability;
    /// Floored accumulation (never below EPSILON).
    fn learn(accumulated: Probability, immediate: Probability, epoch: usize) -> Probability {
        Self::accumulate(accumulated, immediate, epoch).max(EPSILON)
    }
}
