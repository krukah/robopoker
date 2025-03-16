use super::edge::Edge;
use super::edges::EdgeSet;
use super::game::Game;
use super::leaf::Leaf;
use super::node::Node;
use super::nodes::NodeSet;

/// Trees handle the recursive growth of their
/// underlying graph data structure. All we need
/// is a root (Node) and some forks (Edge, Node)
/// to recursively insert new Games.
pub trait Tree: Clone + Copy + Default {
    /// Once the Tree has been fully constructed,
    /// we Partition it into a set of InfoSets, which
    /// themselves are sets of Nodes. After partitioning,
    /// the Tree exists by Arc<_> reference and is dropped
    /// when the "batch" of Trees is collected into a
    /// Vec<Counterfactual> i.e. Iterator<Item = (I, R, P)>
    /// where I: Info, R: Density<Support = Edge>, P: Density<Support = Edge>
    fn partition<N, I, J>(self) -> J
    where
        N: Node,
        I: NodeSet<N>,
        J: Iterator<Item = I>;

    /// Initial insertion of root node
    /// must have nothing to do with
    /// topology.
    fn seed<N, I, G, E>(&mut self, info: I, seed: G) -> N
    where
        E: Edge,
        I: EdgeSet<E>,
        G: Game,
        N: Node;

    /// This is the only method by which
    /// the tree can grow, is if we know:
    /// what Game we're attaching,
    /// what Node to attach a new Game,
    /// what Edge to connect it with
    fn grow<N, I, L, E>(&mut self, info: I, leaf: L) -> N
    where
        E: Edge,
        I: EdgeSet<E>,
        L: Leaf,
        N: Node;
}
