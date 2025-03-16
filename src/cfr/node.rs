use super::edge::Edge;
use super::edges::EdgeSet;
use super::game::Game;
use super::tree::Tree;
use super::turn::Turn;

/// Local topology accessible via walkable tree functions.
/// Interior data is exposed via game reference. Topology is
/// exposed via tree reference. This trait only handles
/// navigation methods.
pub trait Node: Clone + Copy + PartialEq + Eq {
    /// Lookup the pre-computed information
    fn info<E, I>(&self) -> &I
    where
        E: Edge,
        I: EdgeSet<E>;

    /// Reveal interior data storage
    /// by reference, assuming it cannot
    /// change after tree insertion.
    fn game<G>(&self) -> &G
    where
        G: Game;

    /// Reveal enclosing tree by reference. We can probably assume
    /// that the implementation will yield a Node that is
    /// owned by this Tree Graph, and thus we would
    /// also have some lifetime parameters.
    fn tree<T>(&self) -> &T
    where
        T: Tree;

    /// exposing through self.game implicitly
    fn turn<T>(&self) -> &T
    where
        T: Turn;

    /// This will go one step down in the tree
    /// w.r.t. another Edge AND another Node
    fn follow<E>(&self, edge: &E) -> Option<Self>
    where
        E: Edge;

    /// This will go one step up in the tree
    /// w.r.t. another Edge AND another Node
    fn parent<E>(&self) -> Option<(Self, E)>
    where
        E: Edge;

    /// outgoing edges
    fn outgoing<E, I>(&self) -> I
    where
        E: Edge,
        I: Iterator<Item = E>;

    /// incoming edge optinoal
    fn incoming<E>(&self) -> Option<E>
    where
        E: Edge;

    /// This will go one step down in the tree
    /// w.r.t. all other Nodes
    fn children<I>(&self) -> I
    where
        I: Iterator<Item = Self>;

    /// This will go down as many steps as possible,
    /// recursing through children until we reach terminal nodes
    fn descendants<I>(&self) -> I
    where
        I: Iterator<Item = Self>;

    /// This will go up as many steps as possible,
    /// recursing through parents until we reach the root node
    fn ancestors<E, I>(&self) -> I
    where
        E: Edge,
        I: Iterator<Item = (Self, E)>;
}
