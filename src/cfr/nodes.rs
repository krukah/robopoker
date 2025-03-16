use super::edge::Edge;
use super::node::Node;

/// InfoSet represents a collection of nodes that share the same information state.
/// It combines the properties of being an iterator over nodes with the Info trait.
pub trait Position<N>: Iterator<Item = N> + Clone
where
    N: Node,
{
    fn decision<E, I>(&self) -> &I
    where
        E: Edge,
        I: Iterator<Item = E>;
}
