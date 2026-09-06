//! World-tagged information set for subgame solving.
use super::*;
use mccfr::*;

/// An info set tagged with its [`World`]. Every method delegates inward; the
/// tag only affects identity, so the same inner info under two worlds is two
/// distinct info sets accumulating independent regrets and weights.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldInfo<I>(World, I)
where
    I: CfrInfo;

impl<I> WorldInfo<I>
where
    I: CfrInfo,
{
    pub fn new(world: World, inner: I) -> Self {
        Self(world, inner)
    }

    pub fn world(&self) -> World {
        self.0
    }

    pub fn inner(&self) -> I {
        self.1
    }
}

impl<I> CfrInfo for WorldInfo<I>
where
    I: CfrInfo,
{
    type X = I::X;
    type Y = I::Y;
    type E = I::E;
    type T = I::T;

    fn public(&self) -> Self::X {
        self.1.public()
    }

    fn secret(&self) -> Self::Y {
        self.1.secret()
    }

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<I> {
        self.1.choices()
    }

    fn history(&self) -> Vec<Self::E> {
        self.1.history()
    }
}
