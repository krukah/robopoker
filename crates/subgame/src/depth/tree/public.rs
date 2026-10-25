//! Public state for frontier-augmented games.
use crate::*;
use mccfr::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepthPublic<X, const D: usize>
where
    X: CfrPublic,
{
    Game(X),
    Pick(X),
}

impl<X, const D: usize> DepthPublic<X, D>
where
    X: CfrPublic,
{
    pub fn inner(&self) -> X {
        match self {
            Self::Game(x) | Self::Pick(x) => *x,
        }
    }
}

impl<X, const D: usize> CfrPublic for DepthPublic<X, D>
where
    X: CfrPublic,
{
    type G = DepthGame<X::G, D>;
    fn menu(&self) -> impl Iterator<Item = <Self::G as CfrGame>::E> + use<X, D> {
        match self {
            Self::Game(x) => x.menu().map(DepthEdge::Game).collect::<Vec<_>>().into_iter(),
            Self::Pick(_) => Continuation::all::<D>()
                .map(DepthEdge::Pick)
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    fn count(&self) -> usize {
        match self {
            Self::Game(x) => x.count(),
            Self::Pick(_) => D,
        }
    }
}

impl<I, const D: usize> Unwrap<I> for DepthInfo<I, D>
where
    I: CfrInfo,
{
    fn inner(&self) -> I {
        I::join(self.public().inner(), self.secret())
    }
}
