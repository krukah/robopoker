/// A pending child not yet added to the tree: the edge taken, the game state it
/// produced, and the parent's index.
///
/// `CfrEncoder::info` reads it to compute the new node's information set;
/// `Tree::grow` materializes it as a tree node.
pub type Leaf<E, G> = (E, G, petgraph::graph::NodeIndex);
