use crate::cfr::edge::Edge;
use crate::cfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::ptr::NonNull;

pub struct Info {
    roots: Vec<NodeIndex>,
    graph: NonNull<DiGraph<Node, Edge>>,
}

impl Info {
    pub fn push(&mut self, index: NodeIndex) {
        self.roots.push(index)
    }
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

impl From<(NodeIndex, NonNull<DiGraph<Node, Edge>>)> for Info {
    fn from((index, graph): (NodeIndex, NonNull<DiGraph<Node, Edge>>)) -> Self {
        let roots = vec![index];
        Self { roots, graph }
    }
}
