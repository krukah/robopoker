use crate::*;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
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
    graph: &'tree DiGraph<(G, I), E>,
    danny: std::marker::PhantomData<(T, I)>,
}

impl<'tree, T, E, G, I> Node<'tree, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    /// Creates a node handle from an index and graph reference.
    pub fn from(index: NodeIndex, graph: &'tree DiGraph<(G, I), E>) -> Self {
        Self {
            index,
            graph,
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }
    /// The petgraph index of this node.
    pub fn index(&self) -> NodeIndex {
        self.index
    }
    /// Reference to the underlying graph.
    pub fn graph(&self) -> &'tree DiGraph<(G, I), E> {
        self.graph
    }
    /// Unchecked access to node weight via raw_nodes slice.
    fn weight(&self) -> &(G, I) {
        unsafe {
            &self
                .graph
                .raw_nodes()
                .get_unchecked(self.index.index())
                .weight
        }
    }
    /// The game state at this node.
    pub fn game(&self) -> &G {
        &self.weight().0
    }
    /// The information set identifier at this node.
    pub fn info(&self) -> &I {
        &self.weight().1
    }
    /// Creates a node handle at a different index in the same tree.
    pub fn at(&self, index: NodeIndex) -> Node<'tree, T, E, G, I> {
        Self {
            index,
            graph: self.graph(),
            danny: std::marker::PhantomData::<(T, I)>,
        }
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
    /// Child reached by taking a specific edge.
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
            0 => vec![self.clone()],
            _ => self.children().iter().flat_map(Self::descendants).collect(),
        }
    }
    /// Computes child branches: (edge, resulting game, this index).
    pub fn branches(&self) -> Vec<Branch<E, G>> {
        self.info()
            .choices()
            .into_iter()
            .map(|e| (e.clone(), self.game().apply(e), self.index()))
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
            .take_while(|(p, _)| p.game().turn().is_chance().not())
            .count()
    }
}

/// Node naturally implements Iterator by recursing upward through its tree.
/// Each iteration yields a tuple of (Node, Edge) representing the parent node
/// and the edge taken to reach the current node. This allows traversing
/// from any node back to the root of the tree.
impl<'tree, T, E, G, I> Iterator for Node<'tree, T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    type Item = (Self, E);
    fn next(&mut self) -> Option<Self::Item> {
        let (ref mut parent, edge) = self.up()?;
        std::mem::swap(self, parent);
        Some((self.clone(), edge.clone()))
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
        write!(
            f,
            "{:?} ({}/{})",
            self.info(),
            self.index().index(),
            self.graph().node_count()
        )
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
        self.index() == other.index() && std::ptr::eq(self.graph(), other.graph())
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
