use crate::*;
use petgraph::graph::NodeIndex;

/// A node's children before any of them is built: the game every edge applies to, and the edges.
/// Sampling narrows the edges, and only the ones it keeps become [`Leaf`]s, so an opponent node
/// that samples one action never applies the others.
#[derive(Debug, Clone)]
pub struct Branches<G>
where
    G: CfrGame,
{
    parent: NodeIndex,
    game: G,
    edges: smallvec::SmallVec<[G::E; 16]>,
}

impl<G> Branches<G>
where
    G: CfrGame,
{
    /// `edges` from `game`, which hangs at `parent`.
    pub fn new<I>(parent: NodeIndex, game: G, edges: I) -> Self
    where
        I: IntoIterator<Item = G::E>,
    {
        Self {
            parent,
            game,
            edges: edges.into_iter().collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn edges(&self) -> &[G::E] {
        &self.edges
    }

    fn game(&self) -> &G {
        &self.game
    }

    fn parent(&self) -> NodeIndex {
        self.parent
    }

    /// The child `edge` leads to.
    pub fn leaf(&self, edge: G::E) -> Leaf<G> {
        Leaf::new(edge, self.game().apply(edge), self.parent())
    }

    /// Builds every remaining child, in edge order.
    pub fn leaves(self) -> impl Iterator<Item = Leaf<G>> {
        let Self { parent, game, edges } = self;
        edges
            .into_iter()
            .map(move |edge| Leaf::new(edge, game.apply(edge), parent))
    }

    /// Keeps only the `index`th edge.
    pub fn only(self, index: usize) -> Self {
        Self {
            edges: smallvec::smallvec![self.edges()[index]],
            ..self
        }
    }

    /// Keeps the edges `keep` accepts, in order. `keep` sees these branches, to build a child it
    /// needs to inspect.
    pub fn retain<F>(self, mut keep: F) -> Self
    where
        F: FnMut(&Self, G::E) -> bool,
    {
        let edges = self.edges().iter().copied().filter(|&edge| keep(&self, edge)).collect();
        Self { edges, ..self }
    }
}
