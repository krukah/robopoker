use crate::*;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::ops::Not;

/// A lightweight handle to a node in the game tree.
///
/// Stores only an index and a reference to the underlying graph,
/// making nodes cheap to copy and pass around. Provides navigation
/// methods for tree traversal (parent, children, descendants).
///
/// # Iterator Implementation
///
/// Implements `Iterator` for upward traversal: each `next()` yields
/// the parent node and incoming edge, enabling path reconstruction
/// from any node back to the root.
///
/// # Navigation
///
/// - `up()` — Parent node and incoming edge
/// - `children()` — Direct child nodes
/// - `descendants()` — All leaf nodes reachable from this node
/// - `follow(edge)` — Child reached by taking a specific action
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
    /// Stable identity for the tree this node belongs to.
    /// Assigned explicitly at Tree construction (see [`Tree::new`]); the
    /// caller — typically the par_iter index in `Solver::batch` — provides
    /// a batch-local id so trees in one batch sample independently.
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

/// Node naturally implements Iterator by recursing upward through its tree.
/// Each iteration yields an [`Ascent`] pair: the edge that was just traversed
/// in reverse, paired with the parent node we've now arrived at. The
/// iterator's direction (leaf-to-root) is encoded in the type: consumers
/// that want a root-to-leaf [`Descent`] sequence must collect + reverse,
/// not silently flip pairs in place.
impl<'tree, T, E, G, I> Iterator for Node<'tree, T, E, G, I>
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

/// Debug + Display implementations, which will
/// treat a Node just as a combination of its
/// associated Info + its location in the tree
impl<'tree, T, E, G, I> std::fmt::Debug for Node<'tree, T, E, G, I>
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

/// Eq implementation will assume that any two
/// Nodes being compared to one another belong
/// to the same tree/graph. such that, we only
/// care about comparing indices.
impl<'tree, T, E, G, I> PartialEq for Node<'tree, T, E, G, I>
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
impl<'tree, T, E, G, I> Eq for Node<'tree, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
}
