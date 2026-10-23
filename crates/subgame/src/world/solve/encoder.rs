//! Encoding wrapper for world-tagged subgame info sets.
use crate::*;
use mccfr::*;

/// Encoding that tags inner info sets with a world index.
///
/// During subgame solving, the solver mutates `world` before each batch to select which world's
/// tree is being traversed. All info sets produced by this encoder carry that world tag, and the
/// inner public half reads the subgame's prefix after the path.
pub struct WorldEncoder<'blueprint, N>
where
    N: Encoding,
{
    inner: &'blueprint N,
    prefix: Prefix<GameOf<N>>,
    world: World,
}

impl<'blueprint, N> WorldEncoder<'blueprint, N>
where
    N: Encoding,
{
    pub fn new(inner: &'blueprint N, prefix: Prefix<GameOf<N>>, world: World) -> Self {
        Self { inner, prefix, world }
    }

    pub fn inner(&self) -> &N {
        self.inner
    }

    pub fn prefix(&self) -> &Prefix<GameOf<N>> {
        &self.prefix
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

impl<N> CfrSpec for WorldEncoder<'_, N>
where
    N: Encoding,
{
    type F = WorldInfo<InfoOf<N>>;
}

impl<N> Encoding for WorldEncoder<'_, N>
where
    N: Encoding,
{
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        WorldPublic::new(
            self.world(),
            self.inner()
                .public(game, past.into_iter().chain(self.prefix().nearest())),
        )
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        self.inner().secret(game)
    }
    /// Blocks chance and terminal nodes from expanding, UNLESS
    /// the game reports `is_frontier()` (depth-limited frontier that
    /// should expand into continuation-choice nodes).
    fn branches(&self, node: &Node<Self::F>) -> Branches<GameOf<Self>> {
        let turn = node.game().turn();
        if turn.is_terminal() || (turn.is_chance() && !node.game().is_frontier()) {
            Branches::new(node.index(), *node.game(), std::iter::empty())
        } else {
            node.branches()
        }
    }
}
