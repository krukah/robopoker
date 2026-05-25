//! CfrEncoder wrapper for world-tagged subgame info sets.
//!
//! Wraps an inner encoder and tags all info sets with the current world.
//! The prefix history is passed to the inner encoder's `resume()` method
//! for correct reach probability computation.
use crate::*;
use rbp_mccfr::*;

/// CfrEncoder that tags inner info sets with a world index.
///
/// During subgame solving, the solver mutates `world` before each batch
/// to select which world's tree is being traversed. All info sets produced
/// by this encoder carry that world tag.
///
/// # Encoding Strategy
///
/// All three methods delegate to `inner.resume(prefix, game)`:
/// - **`seed`**: Encodes the entry game state with prefix context
/// - **`info`**: Encodes child states during tree expansion
/// - **`resume`**: Prepends prefix to the given history
///
/// The `branches` method uses the default (delegates to `node.branches()`).
/// No rejection sampling, no phase dispatch — just world-tagged delegation.
pub struct WorldEncoder<'blueprint, N>
where
    N: CfrEncoder,
{
    inner: &'blueprint N,
    prefix: Vec<Descent<N::T, N::E>>,
    world: World,
}

impl<'blueprint, N> WorldEncoder<'blueprint, N>
where
    N: CfrEncoder,
{
    pub fn new(inner: &'blueprint N, prefix: Vec<Descent<N::T, N::E>>, world: World) -> Self {
        Self { inner, prefix, world }
    }

    pub fn inner(&self) -> &N {
        self.inner
    }

    pub fn prefix(&self) -> &[Descent<N::T, N::E>] {
        &self.prefix
    }

    pub fn with_world(&mut self, world: World) {
        self.world = world;
    }

    /// Edge-only view of the prefix, for consumers that just want the
    /// downward edge sequence to chain into a `resume` call.
    fn prefix_edges(&self) -> impl Iterator<Item = N::E> + '_ {
        self.prefix.iter().map(|d| d.edge())
    }
}

impl<N> CfrEncoder for WorldEncoder<'_, N>
where
    N: CfrEncoder,
{
    type T = N::T;
    type E = N::E;
    type G = N::G;
    type I = WorldInfo<N::I>;

    fn seed(&self, game: &Self::G) -> Self::I {
        WorldInfo::new(self.world, self.inner.resume(self.prefix_edges(), game))
    }

    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (edge, game, parent): Leaf<Self::E, Self::G>,
    ) -> Self::I {
        let ancestors = tree.at(parent).into_iter().collect::<Vec<_>>();
        debug_assert!(
            ancestors.windows(2).all(|w| w[0].node().index() > w[1].node().index()),
            "upward walk should visit strictly decreasing node indices"
        );
        let mut path = ancestors.into_iter().map(|a| a.edge()).collect::<Vec<_>>();
        path.reverse();
        path.push(edge);
        let full = self.prefix_edges().chain(path);
        WorldInfo::new(self.world, self.inner.resume(full, &game))
    }

    fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        let full = self.prefix_edges().chain(past);
        WorldInfo::new(self.world, self.inner.resume(full, game))
    }
    /// Blocks chance and terminal nodes from expanding, UNLESS
    /// the game reports `is_frontier()` (depth-limited frontier that
    /// should expand into continuation-choice nodes).
    fn branches(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>) -> Vec<Leaf<Self::E, Self::G>> {
        let turn = node.game().turn();
        if turn.is_terminal() {
            vec![]
        } else if turn.is_chance() && !node.game().is_frontier() {
            vec![]
        } else {
            node.branches()
        }
    }
}
