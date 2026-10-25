//! An unattached child: the edge, the state it reaches, and where it hangs.
use crate::*;
use petgraph::graph::NodeIndex;

/// A child node that has been *computed* but not yet *attached* — the edge taken, the game
/// state it reaches, and the index of the parent it hangs from.
///
/// Produced by [`Node::branches`] and by `Encoding::branches`; consumed by [`Tree::grow`]. It
/// was a bare `(E, G, NodeIndex)` tuple, which was positional at both ends of that seam and
/// unnameable as an associated type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leaf<G>
where
    G: CfrGame,
{
    edge: G::E,
    game: G,
    parent: NodeIndex,
}

/// The [`Leaf`] of `X`'s game.
pub type LeafOf<X> = Leaf<GameOf<X>>;

impl<G> Leaf<G>
where
    G: CfrGame,
{
    pub fn new(edge: G::E, game: G, parent: NodeIndex) -> Self {
        Self { edge, game, parent }
    }

    /// The edge taken to reach this leaf.
    pub fn edge(&self) -> G::E {
        self.edge
    }

    /// The state the edge reaches.
    pub fn game(&self) -> G {
        self.game
    }

    /// The index of the node this leaf hangs from.
    pub fn parent(&self) -> NodeIndex {
        self.parent
    }
}
