use rbp_hyperparams::HyperParams;

/// Warmstart parameters for seeding subgame-local CFR profiles from a
/// blueprint profile.
///
/// Read by [`crate::RefProf::warmstart`] when synthesizing
/// iteration-count-agnostic prior weights.
#[derive(HyperParams, Clone, Copy, Debug, PartialEq)]
pub struct WarmstartHyperParams {
    prior_strength: u64,
}

impl WarmstartHyperParams {
    pub fn new(prior_strength: u64) -> Self {
        Self { prior_strength }
    }

    /// Effective number of training iterations the blueprint prior
    /// represents when warmstarting a subgame profile.
    pub fn prior_strength(&self) -> u64 {
        self.prior_strength
    }
}

impl Default for WarmstartHyperParams {
    fn default() -> Self {
        Self {
            // Effective number of training iterations the blueprint prior
            // represents. Sized so subgame's ~50k iterations are
            // comparable in effective weight to the prior, regardless of
            // blueprint depth.
            prior_strength: 1 << 14,
        }
    }
}
