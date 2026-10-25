use super::*;

/// How each iteration's strategy is weighted into the running average.
pub trait WeightSchedule {
    /// Raw accumulation before epsilon floor.
    fn accumulate(net: Probability, new: Probability, epoch: usize) -> Probability;
    /// Floored accumulation (never below EPSILON).
    fn learn(net: Probability, new: Probability, epoch: usize) -> Probability {
        Self::accumulate(net, new, epoch).max(EPSILON)
    }
}
