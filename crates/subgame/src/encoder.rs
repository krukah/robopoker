//! Combined encoder for safe subgame solving + depth-limited frontiers.
use crate::depth::*;
use crate::world::*;
use mccfr::*;

/// Merges [`WorldEncoder`] and [`DepthEncoder`]: tags info sets as
/// `WorldInfo<DepthInfo<I>>`, and expands frontier chance nodes into
/// continuation `Pick` branches while other chance/terminal nodes stay leaves.
pub struct SubGameEncoder<'blueprint, N, const L: usize>
where
    N: DepthSampler<L>,
{
    depth: DepthEncoder<'blueprint, N, L>,
    world: World,
}

impl<'blueprint, N, const L: usize> SubGameEncoder<'blueprint, N, L>
where
    N: DepthSampler<L>,
{
    pub fn new(inner: &'blueprint N, prefix: Prefix<GameOf<N>>, world: World) -> Self {
        Self {
            depth: DepthEncoder::new(inner, prefix),
            world,
        }
    }

    pub fn inner(&self) -> &N {
        self.depth.inner()
    }

    pub fn prefix(&self) -> &Prefix<GameOf<N>> {
        self.depth.prefix()
    }

    pub fn world(&self) -> World {
        self.world
    }

    pub fn with_world(&mut self, world: World) {
        *self.world_mut() = world;
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl<N, const L: usize> CfrSpec for SubGameEncoder<'_, N, L>
where
    N: DepthSampler<L>,
{
    type F = WorldInfo<DepthInfo<InfoOf<N>, L>>;
}

impl<N, const L: usize> Encoding for SubGameEncoder<'_, N, L>
where
    N: DepthSampler<L>,
{
    /// The depth encoder's public half, tagged with the world.
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        WorldPublic::new(self.world(), self.depth.public(game, past))
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        self.depth.secret(game)
    }

    fn branches(&self, node: &Node<Self::F>) -> Branches<GameOf<Self>> {
        self.depth
            .grow(node.index(), *node.game(), node.info().public().inner().inner().menu())
    }
}
