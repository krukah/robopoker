use super::data::Data;
use super::edge::Edge;
use super::tree::Tree;
use crate::mccfr::node::Node;
use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Debug, Clone)]
pub struct Info(pub Vec<NodeIndex>);

impl Info {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn add(&mut self, index: NodeIndex) {
        self.0.push(index)
    }
    pub fn nodes<'tree>(&self, tree: &'tree Tree) -> Vec<Node<'tree>> {
        self.0.iter().copied().map(|i| tree.at(i)).collect()
    }
    pub fn node<'tree>(&self, tree: &'tree Tree) -> Node<'tree> {
        self.0
            .iter()
            .next()
            .copied()
            .map(|i| tree.at(i))
            .expect("non-empty infoset")
    }
    #[allow(dead_code)]
    fn graph<'tree>(&self) -> &'tree DiGraph<Data, Edge> {
        todo!("once Info comes with lifetime this can be implemented trivially")
    }
}
