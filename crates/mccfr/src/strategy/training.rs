use crate::*;
use pokerkit::*;

/// The minimal state an MCCFR iteration needs: who is traversing, how to advance
/// the epoch, and the sampling hyperparameters.
///
/// The CFR math itself lives in [`CfrFlow`], blanket-implemented for
/// `RefProf + CfrSampling`.
pub trait CfrSampling: CfrRule {
    /// who's turn is it?
    fn walker(&self) -> Self::T;
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
