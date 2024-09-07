use crate::cfr::edge::Edge;
use crate::cfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Info {
    roots: Vec<NodeIndex>,
    graph: Rc<RefCell<DiGraph<Node, Edge>>>,
}

impl From<(NodeIndex, Rc<RefCell<DiGraph<Node, Edge>>>)> for Info {
    fn from((index, graph): (NodeIndex, Rc<RefCell<DiGraph<Node, Edge>>>)) -> Self {
        let roots = vec![index];
        Self { roots, graph }
    }
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
            .copied()
            .map(|i| self.graph().node_weight(i).expect("valid node index"))
            .expect("non-empty infoset")
    }
    fn graph(&self) -> &DiGraph<Node, Edge> {
        unsafe { self.graph.as_ptr().as_ref().expect("valid graph") }
    }
}
