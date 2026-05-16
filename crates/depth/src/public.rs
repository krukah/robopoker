//! Public state for frontier-augmented games.
use crate::*;
use rbp_mccfr::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepthPublic<X, const D: usize>
where
    X: CfrPublic,
{
    Game(X),
    Pick(X),
}

impl<X, const D: usize> CfrPublic for DepthPublic<X, D>
where
    X: CfrPublic,
{
    type E = DepthEdge<X::E, D>;
    type T = X::T;

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<X, D> {
        match self {
            Self::Game(x) => x
                .choices()
                .map(DepthEdge::Game)
                .collect::<Vec<_>>()
                .into_iter(),
            Self::Pick(_) => Continuation::all::<D>()
                .map(DepthEdge::Pick)
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    fn subgame(&self) -> Vec<Self::E> {
        match self {
            Self::Game(x) | Self::Pick(x) => x.subgame().into_iter().map(DepthEdge::Game).collect(),
        }
    }
}
