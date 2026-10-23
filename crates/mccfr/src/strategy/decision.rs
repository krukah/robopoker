use crate::*;
use pokerkit::*;

/// One row of the strategy table: an action plus its accumulated mass — the unit
/// strategies are rebuilt from when loading out of the database.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision<E>
where
    E: CfrEdge,
{
    edge: E,
    /// Accumulated probability mass, *not* normalized.
    mass: Probability,
    visits: u32,
    /// Expected value of the information set.
    payoff: Utility,
}

impl<E> Decision<E>
where
    E: CfrEdge,
{
    /// Divides mass by denominator for normalization.
    pub fn normalize(self, denom: Probability) -> Self {
        Self {
            mass: self.mass() / denom,
            ..self
        }
    }
}

impl<E> Decision<E>
where
    E: CfrEdge,
{
    pub fn new(edge: E, mass: Probability, visits: u32, payoff: Utility) -> Self {
        Self {
            edge,
            mass,
            visits,
            payoff,
        }
    }

    pub fn edge(&self) -> &E {
        &self.edge
    }

    pub fn mass(&self) -> Probability {
        self.mass
    }

    pub fn visits(&self) -> u32 {
        self.visits
    }

    pub fn payoff(&self) -> Utility {
        self.payoff
    }
}
