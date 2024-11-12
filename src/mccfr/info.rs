use super::data::Data;
use super::edge::Edge;
use super::tree::Tree;
use crate::mccfr::node::Node;
use petgraph::graph::{DiGraph, NodeIndex};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Info {
    roots: Vec<NodeIndex>,
    nodes: Arc<Tree>,
}

impl From<(Arc<Tree>, Vec<NodeIndex>)> for Info {
    fn from((nodes, roots): (Arc<Tree>, Vec<NodeIndex>)) -> Self {
        Self { roots, nodes }
    }
}

impl Info {
    pub fn add(&mut self, index: NodeIndex) {
        self.roots.push(index);
    }
    pub fn nodes(&self) -> Vec<Node> {
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
    #[allow(dead_code)]
    fn graph(&self) -> &DiGraph<Data, Edge> {
        self.nodes.graph()
    }
}
