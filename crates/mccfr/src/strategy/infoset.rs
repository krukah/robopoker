use crate::*;

/// The tree nodes sharing one information set, since CFR updates regret and
/// policy per info set rather than per node.
///
/// Built by [`Tree::partition()`] after tree generation. Every member shares the
/// same info identifier and therefore the same available actions.
#[derive(Debug)]
pub struct InfoSet<T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<E = E, T = T>,
    I: CfrInfo<E = E, T = T>,
{
    span: Vec<petgraph::graph::NodeIndex>,
    tree: std::sync::Arc<Tree<T, E, G, I>>,
}
impl<T, E, G, I> InfoSet<T, E, G, I>
where
    T: CfrTurn,
    E: CfrEdge,
    G: CfrGame<T = T, E = E>,
    I: CfrInfo<E = E, T = T>,
{
    /// Creates an empty info set backed by the given tree.
    pub fn from(tree: std::sync::Arc<Tree<T, E, G, I>>) -> Self {
        Self { span: Vec::new(), tree }
    }
    /// Adds a node index to this info set.
    pub fn push(&mut self, index: petgraph::graph::NodeIndex) {
        self.span.push(index);
    }
    /// Returns node handles for all nodes in this set.
    pub fn span(&self) -> Vec<Node<'_, T, E, G, I>> {
        self.span.iter().copied().map(|i| self.tree.at(i)).collect()
    }
    /// First node in the set (representative for info lookup).
    pub fn head(&self) -> Node<'_, T, E, G, I> {
        self.tree.at(self.span.first().copied().expect("nodes in info"))
    }
    /// The shared information set identifier.
    pub fn info(&self) -> I {
        *self.head().info()
    }
    /// Reference to the underlying tree.
    pub fn tree(&self) -> &Tree<T, E, G, I> {
        &self.tree
    }
    /// Number of nodes that share this information set.
    pub fn size(&self) -> usize {
        self.span.len()
    }
}
