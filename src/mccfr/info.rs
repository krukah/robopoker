use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Info {
    roots: Vec<NodeIndex>,
    graph: Arc<DiGraph<Node, Edge>>,
}

impl From<(NodeIndex, Arc<DiGraph<Node, Edge>>)> for Info {
    fn from((index, graph): (NodeIndex, Arc<DiGraph<Node, Edge>>)) -> Self {
        let roots = vec![index];
        Self { roots, graph }
    }
}

impl Info {
    pub fn add(&mut self, index: NodeIndex) {
        self.roots.push(index)
    }
    pub fn heads(&self) -> Vec<&Node> {
        self.roots
            .iter()
            .copied()
            .map(|i| self.graph_ref().node_weight(i).expect("valid node index"))
            .collect()
    }
    pub fn node(&self) -> &Node {
        self.roots
            .iter()
            .next()
            .copied()
            .map(|i| self.graph_ref().node_weight(i).expect("valid node index"))
            .expect("non-empty infoset")
    }
    /// SAFETY:
    /// we have logical assurance that lifetimes work out effectively:
    /// 'info: 'node: 'tree
    /// Info is created from a Node
    /// Node is created from a Tree
    /// Tree owns its Graph
    fn graph_ref(&self) -> &DiGraph<Node, Edge> {
        self.graph.as_ref()
    }
}
