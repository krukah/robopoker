use super::*;

/// How accumulated regret is updated — the choice that names the CFR variant.
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
