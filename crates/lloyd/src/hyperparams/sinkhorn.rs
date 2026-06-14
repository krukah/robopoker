use fulcrum::*;

/// Sinkhorn entropic optimal transport parameters.
///
/// These knobs are fixed at compile time. The abstraction artifact is
/// generated once per (Version × Regime) and these values influence its
/// output; changing them at runtime would silently desync the active
/// blueprint from the persisted EMD distances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SinkhornHyperParams {
    temperature: Entropy,
    iterations: usize,
    tolerance: Energy,
}

impl SinkhornHyperParams {
    pub const DEFAULT: Self = Self {
        temperature: 0.025,
        // Calibrated 2026-04-28 (V1 cluster run): ~91% early-term at iter cap
        // 256, bulk converges <128. Cap=256 was paranoia.
        iterations: 128,
        tolerance: 0.0005,
    };

    /// Entropy regularization strength. Lower = closer to true EMD.
    pub fn temperature(&self) -> Entropy {
        self.temperature
    }

    /// Maximum Sinkhorn-Knopp iterations.
    pub fn iterations(&self) -> usize {
        self.iterations
    }

    /// Early-stopping threshold on marginal constraint violation.
    pub fn tolerance(&self) -> Energy {
        self.tolerance
    }
}

impl Default for SinkhornHyperParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}
