use crate::*;
use rbp_core::*;

/// A single action-weight pair from a trained strategy.
///
/// Used as the building block for strategy construction when
/// loading from database. Each decision represents one row from
/// the strategy table: an edge (action) and its accumulated mass.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision<E>
where
    E: CfrEdge,
{
    /// The action this decision represents.
    pub edge: E,
    /// Accumulated probability mass (not normalized).
    pub mass: Probability,
    /// Number of times this action was encountered during training.
    pub counts: u32,
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
            counts: self.counts,
        }
    }
}
