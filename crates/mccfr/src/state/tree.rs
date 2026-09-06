use crate::*;
use petgraph::graph::NodeIndex;

/// A sampled game tree for CFR traversal: a petgraph `DiGraph` whose vertices
/// hold `(Game, Info)` and whose edges are labeled with actions.
///
/// Grown depth-first over a single training iteration, then `partition()`ed by
/// information set for regret computation.
#[derive(Debug)]
pub struct Tree<T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    id: usize,
    graph: petgraph::graph::DiGraph<(G, I), E>,
    danny: std::marker::PhantomData<(T, I)>,
}

impl<T, E, G, I> Tree<T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    /// Construct an empty tree with the given batch-local identifier.
    ///
    /// [`Node::seed`] uses `id` to distinguish trees within a batch at the same
    /// epoch, so trees needing independent sampling must get distinct ids.
    /// [`Solver::batch`] passes the par_iter index; single-tree tests can pass
    /// anything.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            graph: petgraph::graph::DiGraph::default(),
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }
    /// Number of nodes in the tree.
    pub fn n(&self) -> usize {
        self.graph.node_count()
    }
    /// The tree's batch-local identifier (see [`Tree::new`]).
    pub fn id(&self) -> usize {
        self.id
    }
    /// get all Nodes in the Tree
    pub fn all(&self) -> impl Iterator<Item = Node<'_, T, E, G, I>> {
        self.graph.node_indices().map(|n| self.at(n))
    }
    /// Reference to the underlying petgraph DiGraph.
    pub fn graph(&self) -> &petgraph::graph::DiGraph<(G, I), E> {
        &self.graph
    }
    /// get a Node by index
    pub fn at(&self, index: petgraph::graph::NodeIndex) -> Node<'_, T, E, G, I> {
        Node::from(index, self)
    }
    /// seed a Tree by giving an (Info, Game) and getting a Node
    pub fn seed(&mut self, info: I, seed: G) -> Node<'_, T, E, G, I> {
        let seed = self.graph.add_node((seed, info));
        self.at(seed)
    }
    /// extend a Tree by giving a Leaf and getting a Node
    pub fn grow(&mut self, info: I, leaf: Leaf<E, G>) -> Node<'_, T, E, G, I> {
        let tail = self.graph.add_node((leaf.1, info));
        let edge = self.graph.add_edge(leaf.2, tail, leaf.0);
        debug_assert_eq!(edge.index(), tail.index() - 1);
        self.at(tail)
    }
    /// group non-leaf Nodes by Info into InfoSets
    pub fn partition(self) -> std::collections::HashMap<I, InfoSet<T, E, G, I>> {
        let tree = std::sync::Arc::new(self);
        let mut info = std::collections::HashMap::new();
        for node in tree.all().filter(|n| n.width() > 0) {
            info.entry(*node.info())
                .or_insert_with(|| InfoSet::from(tree.clone()))
                .push(node.index());
        }
        info
    }

    /// Nodes in BFS order — parents before children, for top-down traversal.
    pub fn bfs(&self) -> Vec<NodeIndex> {
        use petgraph::visit::Walker;
        petgraph::visit::Bfs::new(&self.graph, NodeIndex::new(0))
            .iter(&self.graph)
            .collect()
    }
    /// Nodes in postorder — leaves first, for bottom-up value propagation.
    pub fn postorder(&self) -> Vec<NodeIndex> {
        let mut result = Vec::with_capacity(self.n());
        let mut stack = vec![(NodeIndex::new(0), false)];
        while let Some((node, expanded)) = stack.pop() {
            if expanded {
                result.push(node);
            } else {
                stack.push((node, true));
                for child in self.at(node).children() {
                    stack.push((child.index(), false));
                }
            }
        }
        result
    }
    /// display the Tree in a human-readable format
    /// be careful because it's really big and recursive
    fn show(&self, f: &mut std::fmt::Formatter, x: NodeIndex, prefix: &str) -> std::fmt::Result {
        if x == NodeIndex::new(0) {
            writeln!(f, "\nROOT   {:?}", self.at(x).info())?;
        }
        let children = self.graph.neighbors_directed(x, petgraph::Outgoing).collect::<Vec<_>>();
        let n = children.len();
        for (i, child) in children.into_iter().rev().enumerate() {
            let last = i == n - 1;
            let gaps = if last { "    " } else { "│   " };
            let stem = if last { "└" } else { "├" };
            let node = self.at(child);
            let head = node.info();
            let edge = self.graph.edge_weight(self.graph.find_edge(x, child).unwrap()).unwrap();
            writeln!(f, "{prefix}{stem}──{edge:?} → {head:?}")?;
            self.show(f, child, &format!("{prefix}{gaps}"))?;
        }
        Ok(())
    }
}

impl<T, E, G, I> std::fmt::Display for Tree<T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    I: CfrInfo<E = E, T = T>,
    G: CfrGame<E = E, T = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.show(f, NodeIndex::new(0), "")
    }
}
