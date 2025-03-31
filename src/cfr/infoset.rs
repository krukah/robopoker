use super::edge::Edge;
use super::game::Game;
use super::info::Info;
use super::node::Node;
use super::tree::Tree;
use super::turn::Turn;

/// the infoset is pre-implemented. it is an [un]ordered collection of
/// Node indices, and a thread-safe readonly reference to the Tree
/// in which they reside.
///
/// because Node's preserve lifetime and full-tree reference,
/// we are able to walk in and out of this InfoSet without
/// having to allocate anything real. Node's are cheap.
#[derive(Debug, Default)]
pub struct InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    root: Vec<petgraph::graph::NodeIndex>,
    tree: std::sync::Arc<Tree<T, E, G, I>>,
}
impl<T, E, G, I> InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<T = T, E = E>,
    I: Info<E = E, T = T>,
{
    pub fn from(tree: std::sync::Arc<Tree<T, E, G, I>>) -> Self {
        Self {
            root: Vec::new(),
            tree,
        }
    }
    pub fn next(&mut self) -> Option<Node<T, E, G, I>> {
        self.root.pop().map(|i| self.tree.at(i))
    }
    pub fn push(&mut self, index: petgraph::graph::NodeIndex) {
        self.root.push(index);
    }
    pub fn span(&self) -> Vec<Node<T, E, G, I>> {
        self.root.iter().copied().map(|i| self.tree.at(i)).collect()
    }
    pub fn head(&self) -> Node<T, E, G, I> {
        self.tree
            .at(self.root.first().copied().expect("nodes in info"))
    }
    pub fn info(&self) -> I {
        self.head().info().clone()
    }
}
