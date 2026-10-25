use crate::*;
use mccfr::*;

/// The inner public half, tagged with the world it was sampled in. Menus are the inner ones.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldPublic<X>(World, X)
where
    X: CfrPublic;

impl<X> WorldPublic<X>
where
    X: CfrPublic,
{
    pub fn new(world: World, inner: X) -> Self {
        Self(world, inner)
    }

    pub fn world(&self) -> World {
        self.0
    }

    pub fn inner(&self) -> X {
        self.1
    }
}

impl<X> CfrPublic for WorldPublic<X>
where
    X: CfrPublic,
{
    type G = X::G;

    fn menu(&self) -> impl Iterator<Item = <Self::G as CfrGame>::E> + use<X> {
        self.1.menu()
    }

    fn count(&self) -> usize {
        self.1.count()
    }
}

impl<I> Unwrap<I> for WorldInfo<I>
where
    I: CfrInfo,
{
    fn inner(&self) -> I {
        I::join(self.public().inner(), self.secret())
    }
}
