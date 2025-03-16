use super::edge::Edge;
use super::edges::Decision;
use super::game::Game;
use super::leaf::Leaf;
use super::tree::Tree;

/// All of our fat lookup tables are encapsulated
/// by the Encoder. Isomorphism -> Abstraction lookups
/// and Info generation all happen here.
pub trait Encoder {
    /// Lookup the infoset for a game state that
    /// will become the root of the Tree. You don't
    /// need a Tree because you know the topology of the
    /// empty Tree you're going to grow later.
    fn root<D, E, G>(&self, game: &G) -> D
    where
        E: Edge,
        D: Decision<E>,
        G: Game;

    /// For non-root Nodes, we should look at *where* we are
    /// attaching to the Tree to decide *how* we should attach it.
    /// (if depth-limited, or action-constrained, etc.)
    fn info<D, T, E, L>(&self, tree: &T, leaf: &L) -> D
    where
        E: Edge,
        D: Decision<E>,
        T: Tree,
        L: Leaf;
}
