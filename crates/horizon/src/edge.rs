//! Edge type for depth-limited frontier-augmented games.
use crate::*;
use monge::Support;
use pokerkit::*;
use regret::*;

/// Edge type that extends the base game edges with continuation choices.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepthEdge<E, const D: usize>
where
    E: CfrEdge,
{
    /// A regular game action.
    Game(E),
    /// A continuation strategy choice at the depth limit.
    Pick(Continuation),
}

impl<E, const D: usize> Support for DepthEdge<E, D> where E: CfrEdge {}
impl<E, const D: usize> CfrEdge for DepthEdge<E, D>
where
    E: CfrEdge,
{
    fn default_policy(&self) -> Probability {
        match self {
            Self::Game(e) => e.default_policy(),
            Self::Pick(_) => 1.0 / D as Probability,
        }
    }

    fn default_regret(&self) -> Utility {
        match self {
            Self::Game(e) => e.default_regret(),
            Self::Pick(_) => 0.0,
        }
    }
}
