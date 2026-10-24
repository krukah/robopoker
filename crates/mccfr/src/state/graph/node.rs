use crate::*;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::ops::Not;

/// A `Copy` handle to a node in the game tree: just an index plus a borrow of
/// the graph, so it is cheap to pass around.
///
/// Also an `Iterator` walking *upward* to the root — see the impl below.
#[derive(Clone, Copy)]
pub struct Node<'tree, F>
where
    F: CfrInfo,
{
    index: NodeIndex,
    tree: &'tree Tree<F>,
}

impl<'tree, F> Node<'tree, F>
where
    F: CfrInfo,
{
    /// Creates a node handle from an index and its owning tree.
    pub fn from(index: NodeIndex, tree: &'tree Tree<F>) -> Self {
        Self { index, tree }
    }
    /// The petgraph index of this node.
    pub fn index(&self) -> NodeIndex {
        self.index
    }
    fn tree(&self) -> &'tree Tree<F> {
        self.tree
    }
    /// Reference to the underlying graph.
    pub fn graph(&self) -> &'tree DiGraph<(GameOf<F>, F), EdgeOf<F>> {
        self.tree.graph()
    }
    /// Stable identity for the tree this node belongs to, assigned at
    /// construction (see [`Tree::new`]). `Solver::batch` passes the par_iter
    /// index so trees within one batch sample independently.
    pub fn seed(&self) -> usize {
        self.tree.id()
    }
    /// Unchecked access to node weight via raw_nodes slice.
    fn raw(&self) -> &'tree (GameOf<F>, F) {
        let petgraph::graph::Node { weight, .. } =
            unsafe { self.graph().raw_nodes().get_unchecked(self.index().index()) };
        weight
    }
    /// The game state at this node.
    pub fn game(&self) -> &'tree GameOf<F> {
        &self.raw().0
    }
    /// The information set identifier at this node.
    pub fn info(&self) -> &'tree F {
        &self.raw().1
    }
    /// Creates a node handle at a different index in the same tree.
    pub fn at(&self, index: NodeIndex) -> Node<'tree, F> {
        Self {
            index,
            tree: self.tree(),
        }
    }
    /// Returns parent node and incoming edge, if not at root.
    pub fn up(&self) -> Option<(Node<'tree, F>, &'tree EdgeOf<F>)> {
        match (self.parent(), self.incoming()) {
            (None, None) => None,
            (Some(parent), Some(incoming)) => Some((parent, incoming)),
            (Some(_), _) => unreachable!("tree property violation"),
            (_, Some(_)) => unreachable!("tree property violation"),
        }
    }
    /// Parent node (None if this is the root).
    pub fn parent(&self) -> Option<Node<'tree, F>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|index| self.at(index))
    }
    /// The edge taken to reach this node from its parent.
    pub fn incoming(&self) -> Option<&'tree EdgeOf<F>> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(Self::weight)
    }
    /// Iterator over (child_index, &edge_weight) — zero allocation.
    pub fn edges(&self) -> impl Iterator<Item = (NodeIndex, &'tree EdgeOf<F>)> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|e| (e.target(), e.weight()))
    }
    /// Find child by matching outgoing edge weight — no intermediate Vec.
    pub fn step(&self, edge: &EdgeOf<F>) -> Option<Node<'tree, F>> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .find(|e| e.weight() == edge)
            .map(|e| self.at(e.target()))
    }
    /// Child reached by taking a specific edge.
    #[deprecated]
    pub fn follow(&self, edge: &EdgeOf<F>) -> Option<Node<'tree, F>> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.at(child.index()))
    }
    /// All outgoing edges from this node.
    pub fn outgoing(&self) -> Vec<&'tree EdgeOf<F>> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .map(Self::weight)
            .collect()
    }
    fn weight(edge: petgraph::graph::EdgeReference<'tree, EdgeOf<F>>) -> &'tree EdgeOf<F> {
        edge.weight()
    }
    /// All direct child nodes.
    pub fn children(&self) -> Vec<Node<'tree, F>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|index| self.at(index))
            .collect()
    }
    /// All leaf nodes reachable from this node (recursive).
    pub fn descendants(&self) -> Vec<Node<'tree, F>> {
        match self.width() {
            0 => vec![*self],
            _ => self.children().iter().flat_map(Self::descendants).collect(),
        }
    }
    /// This node's children, unbuilt: its info's menu, applied to its game on demand.
    pub fn branches(&self) -> Branches<GameOf<F>> {
        Branches::new(self.index(), *self.game(), self.info().menu())
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
    /// Upward walk yielding only decision points, skipping chance nodes. Dual of
    /// [`Encoding::replay`], which walks downward.
    pub fn decisions(self) -> impl Iterator<Item = Choice<F>> + 'tree
    where
        TurnOf<F>: 'tree,
    {
        self.into_iter()
            .filter(|a| !a.node().game().turn().is_chance())
            .map(|Ascent(e, p)| Choice::new(p.game().turn(), *p.info(), e))
    }
}

/// Recurses upward through the tree, yielding an [`Ascent`]: the edge just
/// traversed in reverse plus the parent now arrived at.
///
/// Leaf-to-root direction is encoded in the item type, so consumers wanting a
/// root-to-leaf [`Descent`] sequence must collect + reverse rather than
/// silently flipping pairs in place.
impl<F> Iterator for Node<'_, F>
where
    F: CfrInfo,
{
    type Item = Ascent<EdgeOf<F>, Self>;

    fn next(&mut self) -> Option<Self::Item> {
        let (ref mut parent, edge) = self.up()?;
        std::mem::swap(self, parent);
        Some(Ascent(*edge, *self))
    }
}

/// Renders a Node as its Info plus its location in the tree.
impl<F> std::fmt::Debug for Node<'_, F>
where
    F: CfrInfo,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ({}/{})", self.info(), self.index().index(), self.graph().node_count())
    }
}

/// Compares indices only — assumes both Nodes belong to the same tree.
impl<F> PartialEq for Node<'_, F>
where
    F: CfrInfo,
{
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }
}
impl<F> Eq for Node<'_, F> where F: CfrInfo {}
