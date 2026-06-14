use crate::CfrEdge;
use fulcrum::*;

/// First-class representation of accumulated CFR data for an info-action pair.
///
/// Replaces the raw `(Probability, Utility)` tuple with a semantic struct that
/// also tracks expected value, enabling depth-limited search and subgame solving.
///
/// # Fields
///
/// - `weight` — Cumulative strategy weight for this action (normalize to get policy)
/// - `regret` — Cumulative counterfactual regret for not taking this action
/// - `payoff` — Expected value of the information set V(I) (stored per action)
/// - `visits` — Number of times this info-action pair has been encountered
///
/// # EV Semantics
///
/// The `payoff` field stores the cumulative (uniformly accumulated) expected value
/// of the information set V(I). Stored redundantly for each action to enable
/// efficient frontier evaluation. Normalize by `visits` to get average V(I).
#[derive(Debug, Clone, Copy, Default)]
pub struct Encounter {
    pub weight: Probability,
    pub regret: Utility,
    pub payoff: Utility,
    pub visits: u32,
}

impl Encounter {
    /// Create a new encounter with initial values.
    pub fn new(weight: Probability, regret: Utility, payoff: Utility, visits: u32) -> Self {
        Self {
            weight,
            regret,
            payoff,
            visits,
        }
    }
    /// Create encounter from legacy tuple format (payoff and visits default to 0).
    pub fn from_tuple(weight: Probability, regret: Utility) -> Self {
        Self {
            weight,
            regret,
            payoff: Utility::default(),
            visits: 0,
        }
    }
}

impl From<(Probability, Utility)> for Encounter {
    fn from((weight, regret): (Probability, Utility)) -> Self {
        Self::from_tuple(weight, regret)
    }
}

impl From<(Probability, Utility, Utility)> for Encounter {
    fn from((weight, regret, payoff): (Probability, Utility, Utility)) -> Self {
        Self::new(weight, regret, payoff, 0)
    }
}

impl From<(Probability, Utility, Utility, u32)> for Encounter {
    fn from((weight, regret, payoff, visits): (Probability, Utility, Utility, u32)) -> Self {
        Self::new(weight, regret, payoff, visits)
    }
}

impl<E> From<&E> for Encounter
where
    E: CfrEdge,
{
    fn from(edge: &E) -> Self {
        Self::from_tuple(edge.default_policy(), edge.default_regret())
    }
}
