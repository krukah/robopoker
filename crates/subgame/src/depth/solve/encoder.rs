//! Encoding for depth-limited frontier games.
use crate::*;
use mccfr::*;

/// Wraps a [`DepthSampler`] source, intercepting frontier chance nodes to inject continuation-
/// choice branches.
///
/// `prefix` is the action history from game root to subgame entry, prepended to every within-
/// tree path so blueprint lookups land on the info sets they were trained on. Internal player
/// and origin depth are read off [`DepthGame`] rather than stored here.
pub struct DepthEncoder<'blueprint, N, const D: usize>
where
    N: DepthSampler<D>,
{
    inner: &'blueprint N,
    prefix: Prefix<GameOf<N>>,
}

impl<'blueprint, N, const D: usize> DepthEncoder<'blueprint, N, D>
where
    N: DepthSampler<D>,
{
    pub fn new(inner: &'blueprint N, prefix: Prefix<GameOf<N>>) -> Self {
        Self { inner, prefix }
    }

    pub fn inner(&self) -> &N {
        self.inner
    }

    pub fn prefix(&self) -> &Prefix<GameOf<N>> {
        &self.prefix
    }

    /// A node's children: continuation picks at the depth limit, nothing at chance or terminal
    /// nodes, the inner menu elsewhere. A frontier node first takes its continuation payoffs.
    pub fn grow<M>(
        &self,
        index: petgraph::graph::NodeIndex,
        game: DepthGame<GameOf<N>, D>,
        menu: M,
    ) -> Branches<DepthGame<GameOf<N>, D>>
    where
        M: Iterator<Item = EdgeOf<N>>,
    {
        let game = if game.at_frontier() {
            game.to_frontier(self.inner().payoffs(self.prefix(), game.inner(), game.internal()))
        } else {
            game
        };
        if game.is_choosing() {
            Branches::new(index, game, Continuation::all::<D>().map(DepthEdge::Pick).collect::<Vec<_>>())
        } else if game.turn().is_terminal() || game.turn().is_chance() {
            Branches::new(index, game, Vec::new())
        } else {
            Branches::new(index, game, menu.map(DepthEdge::Game).collect::<Vec<_>>())
        }
    }
}

impl<N, const D: usize> CfrSpec for DepthEncoder<'_, N, D>
where
    N: DepthSampler<D>,
{
    type F = DepthInfo<InfoOf<N>, D>;
}

impl<N, const D: usize> Encoding for DepthEncoder<'_, N, D>
where
    N: DepthSampler<D>,
{
    /// The inner public half from the path, then the subgame's prefix, tagged with the phase.
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        let inner = self.inner().public(
            game.inner(),
            past.into_iter()
                .filter_map(DepthEdge::game)
                .chain(self.prefix().nearest()),
        );
        match game.phase() {
            DepthPhase::Delegate => DepthPublic::Game(inner),
            _ => DepthPublic::Pick(inner),
        }
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        self.inner().secret(game.inner())
    }

    fn branches(&self, node: &Node<Self::F>) -> Branches<GameOf<Self>> {
        self.grow(node.index(), *node.game(), node.info().public().inner().menu())
    }
}
