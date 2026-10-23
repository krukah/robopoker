//! Edge type for depth-limited frontier-augmented games.
use crate::*;
use mccfr::*;
use monge::Support;
use pokerkit::*;

/// Base game edges extended with continuation choices.
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

impl<E, const D: usize> DepthEdge<E, D>
where
    E: CfrEdge,
{
    /// The wrapped game edge, if this is not a continuation pick.
    pub fn game(self) -> Option<E> {
        match self {
            Self::Game(e) => Some(e),
            Self::Pick(_) => None,
        }
    }
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
