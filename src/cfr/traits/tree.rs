use super::edge::Edge;
use super::edge::EdgeSet;
use super::game::Game;
use super::head::Head;
use super::leaf::Leaf;
use super::node::Node;
use super::node::NodeSet;

/// Trees handle the recursive growth of their
/// underlying graph data structure. All we need
/// is a root (Node) and some forks (Edge, Node)
/// to recursively insert new Games.
pub trait Tree: Default {
    /// Lookup the pre-computed information
    fn at<N, H>(&self, index: H) -> N
    where
        N: Node,
        H: Head;

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
        J: Iterator<Item = I>,
    {
        todo!(
            "
             wrap self in Arc.
             iterate over all nodes.
             filter by player == walker (walker: T?).
             assert n_children > 0.
             fold into BTreeMap<Bucket, InfoSet>.
             collect values as Vec<InfoSet>"
        )
    }

    /// Initial insertion of root node
    /// must have nothing to do with
    /// topology.
    fn seed<N, D, G, E>(&mut self, info: D, seed: G) -> N
    where
        E: Edge,
        N: Node,
        D: EdgeSet<E>,
        G: Game,
    {
        todo!(
            "
            DiGraph holds (Game, DecisionSet) tuples.
    
            let seed = self.0.add_node((seed, info));
            self.at(seed)"
        )
    }

    /// This is the only method by which
    /// the tree can grow, is if we know:
    /// what Game we're attaching,
    /// what Node to attach a new Game,
    /// what Edge to connect it with
    fn grow<N, D, L, E>(&mut self, info: D, leaf: L) -> N
    where
        E: Edge,
        N: Node,
        D: EdgeSet<E>,
        L: Leaf,
    {
        todo!(
            "
            DiGraph holds (Game, DecisionSet) tuples.
            
            let head = leaf.head().clone();
            let into = leaf.edge().clone();
            let tail = self.0.add_node(leaf.game().clone());
            let edge = self.0.add_edge(head, tail, into);
            self.at(tail)"
        )
    }
}
