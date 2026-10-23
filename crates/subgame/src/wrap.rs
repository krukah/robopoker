use crate::*;
use mccfr::*;

/// Tags an info set for a subgame wrapper. Each tag lands in the public half; the secret half
/// passes through unchanged.
pub trait Wrap: CfrInfo {
    fn game<const D: usize>(self) -> DepthInfo<Self, D> {
        Composite::new(DepthPublic::Game(self.public()), self.secret())
    }

    fn world(self, world: World) -> WorldInfo<Self> {
        Composite::new(WorldPublic::new(world, self.public()), self.secret())
    }

    fn augmented<Off>(self) -> NestInfo<Self, Off> {
        Composite::new(NestPublic::Augmented(self.public()), self.secret())
    }
}

impl<I> Wrap for I where I: CfrInfo {}
