use crate::CfrEdge;
use pokerkit::*;

/// Accumulated CFR data for one info-action pair: strategy `weight` (normalize
/// to get policy), counterfactual `regret`, `payoff`, and `visits`.
///
/// `payoff` is the infoset-level V(I), stored redundantly on every action so
/// frontier evaluation in depth-limited search and subgame solving is a single
/// lookup.
#[derive(Debug, Clone, Copy, Default)]
pub struct Encounter {
    pub weight: Probability,
    pub regret: Utility,
    pub payoff: Utility,
    pub visits: u32,
}

impl Encounter {
    pub fn new(weight: Probability, regret: Utility, payoff: Utility, visits: u32) -> Self {
        Self {
            weight,
            regret,
            payoff,
            visits,
        }
    }
    /// Legacy tuple format; `payoff` and `visits` default to zero.
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
