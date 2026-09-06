//! CfrEncoder for depth-limited frontier games.
use super::*;
use mccfr::*;

/// Wraps a [`DepthSampler`] source, intercepting frontier chance nodes to
/// inject continuation-choice branches.
///
/// `prefix` is the action history from game root to subgame entry, prepended
/// to every within-tree path so blueprint lookups land on the info sets they
/// were trained on. Internal player and origin depth are read off
/// [`DepthGame`] rather than stored here.
pub struct DepthEncoder<'blueprint, N, const D: usize>
where
    N: DepthSampler<D>,
{
    inner: &'blueprint N,
    prefix: Prefix<N::T, N::E>,
}

impl<'blueprint, N, const D: usize> DepthEncoder<'blueprint, N, D>
where
    N: DepthSampler<D>,
{
    pub fn new(inner: &'blueprint N, prefix: Vec<Descent<N::T, N::E>>) -> Self {
        Self {
            inner,
            prefix: Prefix::new(prefix),
        }
    }

    pub fn inner(&self) -> &N {
        self.inner
    }

    fn unwrap<I>(path: I) -> impl Iterator<Item = N::E>
    where
        I: IntoIterator<Item = DepthEdge<N::E, D>>,
    {
        path.into_iter().filter_map(|e| match e {
            DepthEdge::Game(e) => Some(e),
            DepthEdge::Pick(_) => None,
        })
    }

    fn wrap<I>(&self, inner_path: I, game: &DepthGame<N::G, D>) -> DepthInfo<N::I, D>
    where
        I: IntoIterator<Item = N::E>,
    {
        let full = (&self.prefix).edges().chain(inner_path);
        let inner = self.inner.resume(full, game.inner());
        match game.phase() {
            DepthPhase::Delegate => DepthInfo::Game(inner),
            _ => DepthInfo::Pick(inner),
        }
    }
}

impl<N, const D: usize> CfrEncoder for DepthEncoder<'_, N, D>
where
    N: DepthSampler<D>,
{
    type T = N::T;
    type E = DepthEdge<N::E, D>;
    type G = DepthGame<N::G, D>;
    type I = DepthInfo<N::I, D>;

    fn seed(&self, game: &Self::G) -> Self::I {
        self.wrap(std::iter::empty(), game)
    }

    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (edge, game, parent): Leaf<Self::E, Self::G>,
    ) -> Self::I {
        let mut path = tree.at(parent).into_iter().map(mccfr::Jump::edge).collect::<Vec<_>>();
        path.reverse();
        path.push(edge);
        self.wrap(Self::unwrap(path), &game)
    }

    fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        self.wrap(Self::unwrap(past), game)
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
            Continuation::all::<D>()
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
                .choices()
                .map(|e| {
                    let edge = DepthEdge::Game(e);
                    (edge, game.apply(edge), parent)
                })
                .collect()
        }
    }
}
