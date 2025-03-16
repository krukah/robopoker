use super::edge::Edge;

/// This is more like Bucket than it is InfoSet.
/// It's just the copyable struct. It is NOT the
/// collection of Nodes that share the same Bucket.
/// That is going to be something else, part of the Partition.
pub trait EdgeSet<E>: Iterator<Item = E> + Clone + Copy + PartialEq + Eq
where
    E: Edge,
{
}
