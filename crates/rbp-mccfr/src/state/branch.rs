/// A pending child node not yet added to the tree.
///
/// Contains everything needed to expand the game tree:
/// - The edge (action) taken from the parent
/// - The resulting game state after applying that action
/// - The parent's node index for creating the graph edge
///
/// Used by [`Encoder::info`] to compute information sets for new nodes
/// and by [`Tree::grow`] to materialize the branch as a tree node.
pub type Branch<E, G> = (E, G, petgraph::graph::NodeIndex);
