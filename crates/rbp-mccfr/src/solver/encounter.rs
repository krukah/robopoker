use rbp_core::*;

/// First-class representation of accumulated CFR data for an info-action pair.
///
/// Replaces the raw `(Probability, Utility)` tuple with a semantic struct that
/// also tracks expected value, enabling depth-limited search and subgame solving.
///
/// # Fields
///
/// - `weight` — Cumulative strategy weight for this action (normalize to get policy)
/// - `regret` — Cumulative counterfactual regret for not taking this action
/// - `evalue` — Expected value of the information set V(I) (stored per action)
/// - `counts` — Number of times this info-action pair has been encountered
///
/// # EV Semantics
///
/// The `evalue` field stores the expected value of the information set V(I) under
/// the current strategy. It is replaced (not accumulated) on each training update,
/// and stored redundantly for each action to enable efficient frontier evaluation
/// in depth-limited search and safe subgame solving.
#[derive(Debug, Clone, Copy, Default)]
pub struct Encounter {
    pub weight: Probability,
    pub regret: Utility,
    pub evalue: Utility,
    pub counts: u32,
}

impl Encounter {
    /// Create a new encounter with initial values.
    pub fn new(weight: Probability, regret: Utility, evalue: Utility, counts: u32) -> Self {
        Self {
            weight,
            regret,
            evalue,
            counts,
        }
    }
    /// Create encounter from legacy tuple format (evalue and counts default to 0).
    pub fn from_tuple(weight: Probability, regret: Utility) -> Self {
        Self {
            weight,
            regret,
            evalue: Utility::default(),
            counts: 0,
        }
    }
}

impl From<(Probability, Utility)> for Encounter {
    fn from((weight, regret): (Probability, Utility)) -> Self {
        Self::from_tuple(weight, regret)
    }
}

impl From<(Probability, Utility, Utility)> for Encounter {
    fn from((weight, regret, evalue): (Probability, Utility, Utility)) -> Self {
        Self::new(weight, regret, evalue, 0)
    }
}

impl From<(Probability, Utility, Utility, u32)> for Encounter {
    fn from((weight, regret, evalue, counts): (Probability, Utility, Utility, u32)) -> Self {
        Self::new(weight, regret, evalue, counts)
    }
}
