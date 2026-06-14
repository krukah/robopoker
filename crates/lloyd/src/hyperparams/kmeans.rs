use kicker::Street;
use pokerkit::*;

/// K-means clustering parameters.
///
/// These knobs are fixed at compile time. Clustering output is
/// deterministic given (params, hashed seed); changing them at runtime
/// would silently produce a different abstraction than what is persisted
/// in the DB. To change them, bump `Version` and regenerate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KmeansHyperParams {
    flop_iterations: usize,
    turn_iterations: usize,
    drift_threshold: Energy,
}

impl KmeansHyperParams {
    pub const DEFAULT: Self = Self {
        flop_iterations: 32, // was 20
        turn_iterations: 32, // was 24
        // 0.0 disables early termination — pick a real value once drift
        // values from a real run are visible in Grafana.
        drift_threshold: 0.0,
    };

    /// Lloyd's iterations for the given street. Preflop / river return 0
    /// since they are not clustered (preflop is its own bucket per
    /// isomorphism, river is the equity ladder).
    pub fn iterations(&self, street: Street) -> usize {
        match street {
            Street::Pref | Street::Rive => 0,
            Street::Flop => self.flop_iterations,
            Street::Turn => self.turn_iterations,
        }
    }

    /// Stop k-means iteration when largest centroid movement falls below this.
    pub fn drift_threshold(&self) -> Energy {
        self.drift_threshold
    }
}

impl Default for KmeansHyperParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}
