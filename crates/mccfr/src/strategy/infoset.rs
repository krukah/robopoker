use crate::*;

/// The tree nodes sharing one information set, since CFR updates regret and
/// policy per info set rather than per node.
///
/// Built by [`Tree::partition()`] after tree generation. Every member shares the
/// same info identifier and therefore the same available actions.
pub struct InfoSet<F>
where
    F: CfrInfo,
{
    span: Vec<petgraph::graph::NodeIndex>,
    tree: std::sync::Arc<Tree<F>>,
}
impl<F> InfoSet<F>
where
    F: CfrInfo,
{
    /// Creates an empty info set backed by the given tree.
    pub fn from(tree: std::sync::Arc<Tree<F>>) -> Self {
        Self { span: Vec::new(), tree }
    }
    /// Adds a node index to this info set.
    pub fn push(&mut self, index: petgraph::graph::NodeIndex) {
        self.indices_mut().push(index);
    }
    /// Returns node handles for all nodes in this set.
    pub fn span(&self) -> Vec<Node<'_, F>> {
        self.indices().iter().copied().map(|i| self.tree().at(i)).collect()
    }
    /// First node in the set (representative for info lookup).
    pub fn head(&self) -> Node<'_, F> {
        self.tree().at(self.indices().first().copied().expect("nodes in info"))
    }
    /// The shared information set identifier.
    pub fn info(&self) -> F {
        *self.head().info()
    }
    fn indices(&self) -> &[petgraph::graph::NodeIndex] {
        &self.span
    }

    fn indices_mut(&mut self) -> &mut Vec<petgraph::graph::NodeIndex> {
        &mut self.span
    }
    /// Reference to the underlying tree.
    pub fn tree(&self) -> &Tree<F> {
        &self.tree
    }
    /// Number of nodes that share this information set.
    pub fn size(&self) -> usize {
        self.span.len()
    }
}
