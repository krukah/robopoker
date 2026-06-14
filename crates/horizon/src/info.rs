//! Information set for frontier-augmented games.
use crate::*;
use regret::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepthInfo<I, const D: usize>
where
    I: CfrInfo,
{
    Game(I),
    Pick(I),
}

impl<I, const D: usize> DepthInfo<I, D>
where
    I: CfrInfo,
{
    pub fn inner(&self) -> I {
        match self {
            Self::Game(i) | Self::Pick(i) => *i,
        }
    }
}

impl<I, const D: usize> CfrInfo for DepthInfo<I, D>
where
    I: CfrInfo,
{
    type E = DepthEdge<I::E, D>;
    type T = I::T;
    type X = DepthPublic<I::X, D>;
    type Y = I::Y;

    fn public(&self) -> Self::X {
        match self {
            Self::Game(i) => DepthPublic::Game(i.public()),
            Self::Pick(i) => DepthPublic::Pick(i.public()),
        }
    }

    fn secret(&self) -> Self::Y {
        self.inner().secret()
    }
}
