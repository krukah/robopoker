use super::edge::Edge;
use super::edges::EdgeSet;
use super::node::Node;

/// InfoSet represents a collection of nodes that share the same information state.
/// It combines the properties of being an iterator over nodes with the Info trait.
pub trait NodeSet<N>: Iterator<Item = N> + Clone
where
    N: Node,
{
    fn info<E, I>(&self) -> &I
    where
        E: Edge,
        I: EdgeSet<E>;
}
