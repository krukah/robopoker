//! Combined encoder for safe subgame solving + depth-limited frontiers.
//!
//! Merges the responsibilities of [`WorldEncoder`] and [`DepthEncoder`]:
//! tags all info sets with the current world AND detects frontier chance
//! nodes to inject continuation-choice branches.
use atlas::*;
use horizon::*;
use regret::*;

/// Combined encoder for safe subgame solving + depth-limited frontiers.
///
/// Merges the responsibilities of [`WorldEncoder`] and [`DepthEncoder`]:
/// - Tags all info sets with `WorldInfo<DepthInfo<I>>`
/// - At leaf (frontier) chance nodes: produces continuation Pick branches
/// - At non-leaf chance/terminal nodes: produces no branches
/// - At player nodes: produces Game-wrapped branches
pub struct SubGameEncoder<'blueprint, N, const L: usize>
where
    N: DepthSampler<L>,
{
    inner: &'blueprint N,
    prefix: Prefix<N::T, N::E>,
    world: World,
}

impl<'blueprint, N, const L: usize> SubGameEncoder<'blueprint, N, L>
where
    N: DepthSampler<L>,
{
    pub fn new(inner: &'blueprint N, prefix: Vec<Descent<N::T, N::E>>, world: World) -> Self {
        Self {
            inner,
            prefix: Prefix::new(prefix),
            world,
        }
    }

    pub fn inner(&self) -> &N {
        self.inner
    }

    pub fn with_world(&mut self, world: World) {
        self.world = world;
    }

    fn make_info<I>(&self, past: I, game: &DepthGame<N::G, L>) -> WorldInfo<DepthInfo<N::I, L>>
    where
        I: IntoIterator<Item = N::E>,
    {
        let full = (&self.prefix).edges().chain(past);
        let inner = self.inner.resume(full, game.inner());
        let leaf = match game.phase() {
            DepthPhase::Delegate => DepthInfo::Game(inner),
            _ => DepthInfo::Pick(inner),
        };
        WorldInfo::new(self.world, leaf)
    }
}

impl<N, const L: usize> CfrEncoder for SubGameEncoder<'_, N, L>
where
    N: DepthSampler<L>,
{
    type T = N::T;
    type E = DepthEdge<N::E, L>;
    type G = DepthGame<N::G, L>;
    type I = WorldInfo<DepthInfo<N::I, L>>;

    fn seed(&self, game: &Self::G) -> Self::I {
        self.make_info(std::iter::empty(), game)
    }

    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (edge, game, parent): Leaf<Self::E, Self::G>,
    ) -> Self::I {
        let mut path = tree.at(parent).into_iter().map(regret::Jump::edge).collect::<Vec<_>>();
        path.reverse();
        path.push(edge);
        let inner_path = path.into_iter().filter_map(|e| match e {
            DepthEdge::Game(e) => Some(e),
            DepthEdge::Pick(_) => None,
        });
        self.make_info(inner_path, &game)
    }

    fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        let inner_past = past.into_iter().filter_map(|e| match e {
            DepthEdge::Game(e) => Some(e),
            DepthEdge::Pick(_) => None,
        });
        self.make_info(inner_past, game)
    }

    fn branches(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>) -> Vec<Leaf<Self::E, Self::G>> {
        let parent = node.index();
        let game = *node.game();
        let game = if game.at_frontier() {
            let payoffs = self.inner.payoffs(&self.prefix, game.inner(), game.internal());
            game.to_frontier(payoffs)
        } else {
            game
        };
        if game.is_choosing() {
            Continuation::all::<L>()
                .map(|c| {
                    let edge = DepthEdge::Pick(c);
                    (edge, game.apply(edge), parent)
                })
                .collect()
        } else if game.turn().is_terminal() || game.turn().is_chance() {
            vec![]
        } else {
            node.info()
                .inner()
                .inner()
                .choices()
                .map(|e| {
                    let edge = DepthEdge::Game(e);
                    (edge, game.apply(edge), parent)
                })
                .collect()
        }
    }
}
