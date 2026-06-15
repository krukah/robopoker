//! World-tagged information set for subgame solving.
//!
//! Wraps the inner game's info set with a [`World`] tag so that each world
//! accumulates separate regrets and weights during subgame CFR. All methods
//! delegate to the inner info set — the world tag only affects identity.
use crate::*;
use mccfr::*;

/// Information set tagged with its world for per-world regret separation.
///
/// Two `WorldInfo` values with the same inner info but different worlds
/// are distinct info sets, enabling the solver to maintain independent
/// strategies per world.
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
