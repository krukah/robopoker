use crate::*;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::ops::Not;

/// A `Copy` handle to a node in the game tree: just an index plus a borrow of
/// the graph, so it is cheap to pass around.
///
/// Also an `Iterator` walking *upward* to the root — see the impl below.
#[derive(Copy, Clone)]
pub struct Node<'tree, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    index: NodeIndex,
    tree: &'tree Tree<T, E, G, I>,
}

impl<'tree, T, E, G, I> Node<'tree, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    /// Creates a node handle from an index and its owning tree.
    pub fn from(index: NodeIndex, tree: &'tree Tree<T, E, G, I>) -> Self {
        Self { index, tree }
    }
    /// The petgraph index of this node.
    pub fn index(&self) -> NodeIndex {
        self.index
    }
    /// Reference to the underlying graph.
    pub fn graph(&self) -> &'tree DiGraph<(G, I), E> {
        self.tree.graph()
    }
    /// Stable identity for the tree this node belongs to, assigned at
    /// construction (see [`Tree::new`]). `Solver::batch` passes the par_iter
    /// index so trees within one batch sample independently.
    pub fn seed(&self) -> usize {
        self.tree.id()
    }
    /// Unchecked access to node weight via raw_nodes slice.
    fn raw(&self) -> &'tree (G, I) {
        unsafe { &self.graph().raw_nodes().get_unchecked(self.index.index()).weight }
    }
    /// The game state at this node.
    pub fn game(&self) -> &'tree G {
        &self.raw().0
    }
    /// The information set identifier at this node.
    pub fn info(&self) -> &'tree I {
        &self.raw().1
    }
    /// Creates a node handle at a different index in the same tree.
    pub fn at(&self, index: NodeIndex) -> Node<'tree, T, E, G, I> {
        Self { index, tree: self.tree }
    }
    /// Returns parent node and incoming edge, if not at root.
    pub fn up(&self) -> Option<(Node<'tree, T, E, G, I>, &'tree E)> {
        match (self.parent(), self.incoming()) {
            (None, None) => None,
            (Some(parent), Some(incoming)) => Some((parent, incoming)),
            (Some(_), _) => unreachable!("tree property violation"),
            (_, Some(_)) => unreachable!("tree property violation"),
        }
    }
    /// Parent node (None if this is the root).
    pub fn parent(&self) -> Option<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|index| self.at(index))
    }
    /// The edge taken to reach this node from its parent.
    pub fn incoming(&self) -> Option<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|edge| edge.weight())
    }
    /// Iterator over (child_index, &edge_weight) — zero allocation.
    pub fn edges(&self) -> impl Iterator<Item = (NodeIndex, &'tree E)> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|e| (e.target(), e.weight()))
    }
    /// Find child by matching outgoing edge weight — no intermediate Vec.
    pub fn step(&self, edge: &E) -> Option<Node<'tree, T, E, G, I>> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .find(|e| e.weight() == edge)
            .map(|e| self.at(e.target()))
    }
    /// Child reached by taking a specific edge.
    #[deprecated]
    pub fn follow(&self, edge: &E) -> Option<Node<'tree, T, E, G, I>> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.at(child.index()))
    }
    /// All outgoing edges from this node.
    pub fn outgoing(&self) -> Vec<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|edge| edge.weight())
            .collect()
    }
    /// All direct child nodes.
    pub fn children(&self) -> Vec<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|index| self.at(index))
            .collect()
    }
    /// All leaf nodes reachable from this node (recursive).
    pub fn descendants(&self) -> Vec<Node<'tree, T, E, G, I>> {
        match self.width() {
            0 => vec![*self],
            _ => self.children().iter().flat_map(Self::descendants).collect(),
        }
    }
    /// Computes child branches: (edge, resulting game, this index).
    pub fn branches(&self) -> Vec<Leaf<E, G>> {
        self.info()
            .choices()
            .map(|e| (e, self.game().apply(e), self.index()))
            .collect()
    }
    /// Count of direct child nodes (no allocation).
    pub fn width(&self) -> usize {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Outgoing)
            .count()
    }
    /// Actions on current street: count edges up to (but not including) last chance node.
    pub fn depth(&self) -> usize {
        self.into_iter()
            .take_while(|a| a.node().game().turn().is_chance().not())
            .count()
    }
    /// Upward walk yielding only decision points: `(turn, info, edge)`.
    /// Skips chance nodes. Dual of [`CfrEncoder::replay`], which walks downward.
    /// Both yield `(T, I, E)` triples for uniform consumption by reach functions.
    pub fn decisions(self) -> impl Iterator<Item = (T, I, E)> + 'tree
    where
        T: 'tree,
    {
        self.into_iter()
            .filter(|a| !a.node().game().turn().is_chance())
            .map(|Ascent(e, p)| (p.game().turn(), *p.info(), e))
    }
}

/// Recurses upward through the tree, yielding an [`Ascent`]: the edge just
/// traversed in reverse plus the parent now arrived at.
///
/// Leaf-to-root direction is encoded in the item type, so consumers wanting a
/// root-to-leaf [`Descent`] sequence must collect + reverse rather than
/// silently flipping pairs in place.
impl<T, E, G, I> Iterator for Node<'_, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    type Item = Ascent<E, Self>;

    fn next(&mut self) -> Option<Self::Item> {
        let (ref mut parent, edge) = self.up()?;
        std::mem::swap(self, parent);
        Some(Ascent(*edge, *self))
    }
}

/// Renders a Node as its Info plus its location in the tree.
impl<T, E, G, I> std::fmt::Debug for Node<'_, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ({}/{})", self.info(), self.index().index(), self.graph().node_count())
    }
}

/// Compares indices only — assumes both Nodes belong to the same tree.
impl<T, E, G, I> PartialEq for Node<'_, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }
}
impl<T, E, G, I> Eq for Node<'_, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
}
