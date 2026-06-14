use fulcrum::*;
use solus::HyperParams;

/// Average-strategy sampling parameters.
///
/// Biased sampling from cumulative policy:
/// `σ'(a) = max(ε, (τ·σ(a) + β) / (Σσ + β))`
#[derive(HyperParams, Clone, Copy, Debug, PartialEq)]
pub struct SamplingHyperParams {
    temperature: Entropy,
    smoothing: Energy,
    curiosity: Probability,
}

impl SamplingHyperParams {
    pub fn new(temperature: Entropy, smoothing: Energy, curiosity: Probability) -> Self {
        Self {
            temperature,
            smoothing,
            curiosity,
        }
    }

    /// Temperature (T) — controls sampling entropy via policy scaling.
    pub fn temperature(&self) -> Entropy {
        self.temperature
    }

    /// Smoothing (β) — pseudocount added to numerator and denominator.
    pub fn smoothing(&self) -> Energy {
        self.smoothing
    }

    /// Epsilon (ε) — minimum sampling probability floor.
    pub fn curiosity(&self) -> Probability {
        self.curiosity
    }
}

impl Default for SamplingHyperParams {
    fn default() -> Self {
        Self {
            // Temperature (T) — higher → more uniform; lower → more peaked.
            temperature: 1.0,
            // Smoothing (β) — pseudocount pulling sampling toward uniform.
            smoothing: 2.0,
            // Epsilon (ε) — minimum sampling probability floor.
            curiosity: 0.05,
        }
    }
}
