use crate::*;
use petgraph::graph::NodeIndex;

/// A sampled game tree for CFR traversal.
///
/// Built dynamically by the [`Solver`] during training using the
/// [`Encoder`] for state encoding and [`Profile`] for action sampling.
/// Each vertex stores a `(Game, Info)` tuple; edges are labeled with actions.
///
/// # Structure
///
/// Internally wraps a petgraph `DiGraph`. The tree is built depth-first
/// during a single training iteration, then partitioned by information set
/// for regret computation.
///
/// # Traversal
///
/// - `at(index)` — Get a [`Node`] handle at a specific index
/// - `all()` — Iterate over all nodes
/// - `partition()` — Group nodes by information set for CFR updates
/// - `bfs()` / `postorder()` — Ordered traversal for value propagation
#[derive(Debug)]
pub struct Tree<T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
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
    /// Number of nodes in the tree.
    pub fn n(&self) -> usize {
        self.graph.node_count()
    }
    /// get all Nodes in the Tree
    pub fn all(&self) -> impl Iterator<Item = Node<'_, T, E, G, I>> {
        self.graph.node_indices().map(|n| self.at(n))
    }
    /// get a Node by index
    pub fn at(&self, index: petgraph::graph::NodeIndex) -> Node<'_, T, E, G, I> {
        Node::from(index, &self.graph)
    }
    /// seed a Tree by giving an (Info, Game) and getting a Node
    pub fn seed(&mut self, info: I, seed: G) -> Node<'_, T, E, G, I> {
        let seed = self.graph.add_node((seed, info));
        self.at(seed)
    }
    /// extend a Tree by giving a Leaf and getting a Node
    pub fn grow(&mut self, info: I, leaf: Branch<E, G>) -> Node<'_, T, E, G, I> {
        let tail = self.graph.add_node((leaf.1, info));
        let edge = self.graph.add_edge(leaf.2, tail, leaf.0);
        debug_assert!(edge.index() == tail.index() - 1);
        self.at(tail)
    }
    /// group non-leaf Nodes by Info into InfoSets
    pub fn partition(self) -> std::collections::HashMap<I, InfoSet<T, E, G, I>> {
        let tree = std::sync::Arc::new(self);
        let mut info = std::collections::HashMap::new();
        for node in tree.all().filter(|n| n.width() > 0) {
            info.entry(node.info().clone())
                .or_insert_with(|| InfoSet::from(tree.clone()))
                .push(node.index());
        }
        info
    }

    /// Iterate nodes in BFS order (root first) for top-down traversal.
    /// Returns a Vec that visits parents before children.
    pub fn bfs(&self) -> Vec<NodeIndex> {
        use petgraph::visit::Walker;
        petgraph::visit::Bfs::new(&self.graph, NodeIndex::new(0))
            .iter(&self.graph)
            .collect()
    }
    /// Iterate nodes in postorder (leaves first) for bottom-up traversal.
    /// Returns a Vec since we need to reverse the DFS order.
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
        let children = self
            .graph
            .neighbors_directed(x, petgraph::Outgoing)
            .collect::<Vec<_>>();
        let n = children.len();
        for (i, child) in children.into_iter().rev().enumerate() {
            let last = i == n - 1;
            let gaps = if last { "    " } else { "│   " };
            let stem = if last { "└" } else { "├" };
            let node = self.at(child);
            let head = node.info();
            let edge = self
                .graph
                .edge_weight(self.graph.find_edge(x, child).unwrap())
                .unwrap();
            writeln!(f, "{}{}──{:?} → {:?}", prefix, stem, edge, head)?;
            self.show(f, child, &format!("{}{}", prefix, gaps))?;
        }
        Ok(())
    }
}

impl<T, E, G, I> Default for Tree<T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    fn default() -> Self {
        Self {
            graph: petgraph::graph::DiGraph::default(),
            danny: std::marker::PhantomData::<(T, I)>,
        }
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
