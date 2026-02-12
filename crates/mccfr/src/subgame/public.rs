//! Public state for subgame-augmented games.
use super::*;
use crate::*;

/// Public component for subgame info.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubPublic<X, E>
where
    X: CfrPublic<E = E>,
    E: CfrEdge,
{
    Inner(X),
    Root,
}

impl<X, E> CfrPublic for SubPublic<X, E>
where
    X: CfrPublic<E = E>,
    E: CfrEdge,
{
    type E = SubEdge<E>;
    type T = SubTurn<X::T>;
    fn choices(&self) -> Vec<Self::E> {
        match self {
            Self::Inner(x) => x.choices().into_iter().map(SubEdge::Inner).collect(),
            Self::Root => (0..rbp_core::SUBGAME_ALTS).map(SubEdge::World).collect(),
        }
    }
    fn history(&self) -> Vec<Self::E> {
        match self {
            Self::Inner(x) => x.history().into_iter().map(SubEdge::Inner).collect(),
            Self::Root => vec![],
        }
    }
}
