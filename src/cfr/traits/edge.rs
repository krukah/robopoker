/// Represents an edge in the game tree
pub trait Edge: Clone + Copy + PartialEq + Eq + std::fmt::Debug {}

impl Edge for petgraph::graph::EdgeIndex {}
impl Edge for crate::gameplay::edge::Edge {}

/// This is more like Bucket than it is InfoSet.
/// It's just the copyable struct. It is NOT the
/// collection of Nodes that share the same Bucket.
/// That is going to be something else, part of the Partition.
pub trait EdgeSet<E>: Iterator<Item = E> + Clone + Copy + PartialEq + Eq
where
    E: Edge,
{
}

impl EdgeSet<crate::gameplay::edge::Edge> for crate::mccfr::path::Path {}
