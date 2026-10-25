use crate::*;
use pokerkit::*;

/// The clock of an MCCFR run: who traverses at epoch [`Profile::t`], how the epoch advances,
/// and the sampling hyperparameters.
///
/// The CFR math itself lives in [`Flow`], blanket-implemented for
/// `Profile + Clock`.
pub trait Clock: Profile {
    /// Players take turns traversing, one per epoch.
    fn walker(&self) -> TurnOf<Self> {
        <TurnOf<Self>>::from(self.t() % <TurnOf<Self>>::players())
    }
    /// increment epoch
    fn increment(&mut self);
    /// Temperature (T) - controls sampling entropy via policy scaling.
    /// Higher T -> more uniform (exploratory); lower T -> more peaked (greedy).
    fn temperature(&self) -> Entropy {
        SamplingHyperParams::get().temperature()
    }
    /// Smoothing (B) - pseudocount added to numerator and denominator.
    /// Higher values pull sampling toward uniform (maximum entropy prior).
    fn smoothing(&self) -> Energy {
        SamplingHyperParams::get().smoothing()
    }
    /// Epsilon (e) - minimum sampling probability floor.
    /// Ensures every action retains at least e probability for exploration.
    fn curiosity(&self) -> Probability {
        SamplingHyperParams::get().curiosity()
    }
}
