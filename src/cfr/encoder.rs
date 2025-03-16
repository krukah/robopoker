use super::edge::Edge;
use super::edges::EdgeSet;
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
    fn root<I, E, G>(&self, game: &G) -> I
    where
        E: Edge,
        I: EdgeSet<E>,
        G: Game;

    /// For non-root Nodes, we should look at *where* we are
    /// attaching to the Tree to decide *how* we should attach it.
    /// (if depth-limited, or action-constrained, etc.)
    fn info<I, T, E, L>(&self, tree: &T, leaf: &L) -> I
    where
        E: Edge,
        I: EdgeSet<E>,
        T: Tree,
        L: Leaf;
}
