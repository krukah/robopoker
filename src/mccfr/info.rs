use super::node::Node;
use super::tree::Tree;
use petgraph::graph::NodeIndex;
use std::sync::Arc;

/// a lightweight, copyable reference to
/// a set of Nodes within the same sampled Tree
/// that are indistinguishable from the perspective of the
/// player whose action it is! that is, up to a chosen
/// Abstraction space.
///
/// the two constraints for Nodes and InfoSets are:
/// 1. a Node must map to exactly one InfoSet, and
/// 2. any Nodes in the same InfoSet must have the exact same outgoing Edges.
///
/// for this second reason, we make the actual associated
/// Info struct *include* a representation of the collection
/// of outgoing Edges, but also possible more information. e.g. if your only
/// options are Fold, Shove, then there is still a lot to distinguish your
/// position from another (Fold, Shove) positon. both private and public informtion.
/// but we could, if our Abstraction deemed it appropriate, collapse these two Nodes
/// into the same InfoSet, where they would share the same Policy distribution.
#[derive(Debug, Clone)]
pub struct InfoSet {
    roots: Vec<NodeIndex>,
    nodes: Arc<Tree>,
}

impl From<Arc<Tree>> for InfoSet {
    fn from(nodes: Arc<Tree>) -> Self {
        Self {
            roots: vec![],
            nodes,
        }
    }
}

impl InfoSet {
    pub fn add(&mut self, index: NodeIndex) {
        self.roots.push(index);
    }
    pub fn roots(&self) -> Vec<Node> {
        self.roots
            .iter()
            .copied()
            .map(|i| self.nodes.at(i))
            .collect()
    }
    pub fn node(&self) -> Node {
        self.roots
            .iter()
            .next()
            .copied()
            .map(|i| self.nodes.at(i))
            .expect("non-empty infoset")
    }
}
