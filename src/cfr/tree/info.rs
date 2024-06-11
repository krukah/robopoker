use crate::cfr::traits::action::E;
use crate::cfr::tree::node::N;
use petgraph::graph::{DiGraph, NodeIndex};
use std::ptr::NonNull;

pub(crate) struct I {
    pub roots: Vec<NodeIndex>,
    pub graph: NonNull<DiGraph<N, E>>,
}

impl I {
    pub fn add(&mut self, node: &N) {
        self.roots.push(*node.index());
    }
    pub fn roots(&self) -> Vec<&N> {
        self.roots
            .iter()
            .map(|i| self.graph().node_weight(*i).expect("valid node index"))
            .collect()
    }
    pub fn sample(&self) -> &N {
        self.roots
            .iter()
            .next()
            .map(|i| self.graph().node_weight(*i).expect("valid node index"))
            .expect("non-empty infoset")
    }
    fn graph(&self) -> &DiGraph<N, E> {
        unsafe { self.graph.as_ref() }
    }
}
