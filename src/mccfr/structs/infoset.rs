use super::node::Node;
use super::tree::Tree;
use crate::mccfr::traits::edge::Edge;
use crate::mccfr::traits::game::Game;
use crate::mccfr::traits::info::Info;
use crate::mccfr::traits::turn::Turn;

/// the infoset is pre-implemented. it is an [un]ordered collection of
/// Node indices, and a thread-safe readonly reference to the Tree
/// in which they reside.
///
/// because Node's preserve lifetime and full-tree reference,
/// we are able to walk in and out of this InfoSet without
/// having to allocate anything real. Node's are cheap.
///
/// the two constraints for Nodes and InfoSets are:
/// 1. a Node must map to exactly one InfoSet, and
/// 2. any Nodes in the same InfoSet must have the exact same outgoing Edges.
#[derive(Debug, Default)]
pub struct InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    span: Vec<petgraph::graph::NodeIndex>,
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
            span: Vec::new(),
            tree,
        }
    }
    pub fn push(&mut self, index: petgraph::graph::NodeIndex) {
        self.span.push(index);
    }
    pub fn span(&self) -> Vec<Node<'_, T, E, G, I>> {
        self.span.iter().copied().map(|i| self.tree.at(i)).collect()
    }
    pub fn head(&self) -> Node<'_, T, E, G, I> {
        self.tree
            .at(self.span.first().copied().expect("nodes in info"))
    }
    pub fn info(&self) -> I {
        self.head().info().clone()
    }
}
