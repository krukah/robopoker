use crate::*;
use fulcrum::*;

/// Core training state: walker identity and sampling parameters.
///
/// This trait provides the minimal state needed for MCCFR iteration:
/// which player is currently traversing, how to advance epochs, and
/// sampling hyperparameters. The actual CFR math lives in
/// `Counterfactual`, which is blanket-implemented for `Profile + CfrSampling`.
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
