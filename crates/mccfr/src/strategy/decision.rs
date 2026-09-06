use crate::*;
use pokerkit::*;

/// One row of the strategy table: an action plus its accumulated mass — the unit
/// strategies are rebuilt from when loading out of the database.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision<E>
where
    E: CfrEdge,
{
    pub edge: E,
    /// Accumulated probability mass, *not* normalized.
    pub mass: Probability,
    pub visits: u32,
    /// Expected value of the information set.
    pub payoff: Utility,
}

impl<E> Decision<E>
where
    E: CfrEdge,
{
    /// Divides mass by denominator for normalization.
    pub fn normalize(self, denom: Probability) -> Self {
        Self {
            edge: self.edge,
            mass: self.mass / denom,
            visits: self.visits,
            payoff: self.payoff,
        }
    }
}
