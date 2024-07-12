use crate::cfr::tree::rps::action::Edge;
use crate::cfr::tree::rps::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::ptr::NonNull;

pub struct Info {
    pub roots: Vec<NodeIndex>,
    pub graph: NonNull<DiGraph<Node, Edge>>,
}

impl Info {
    pub fn roots(&self) -> Vec<&Node> {
        self.roots
            .iter()
            .map(|i| self.graph().node_weight(*i).expect("valid node index"))
            .collect()
    }
    pub fn node(&self) -> &Node {
        self.roots
            .iter()
            .next()
            .map(|i| self.graph().node_weight(*i).expect("valid node index"))
            .expect("non-empty infoset")
    }
    fn graph(&self) -> &DiGraph<Node, Edge> {
        unsafe { self.graph.as_ref() }
    }
}
