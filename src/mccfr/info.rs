use super::tree::Tree;
use crate::mccfr::node::Node;
use petgraph::graph::NodeIndex;

#[derive(Debug, Clone)]
pub struct Info {
    roots: Vec<NodeIndex>,
}

impl Info {
    pub fn new() -> Self {
        Self { roots: vec![] }
    }
    pub fn add(&mut self, index: NodeIndex) {
        self.roots.push(index)
    }
    pub fn heads<'tree>(&self, tree: &'tree Tree) -> Vec<Node<'tree>> {
        self.roots.iter().copied().map(|i| tree.at(i)).collect()
    }
    pub fn node<'tree>(&self, tree: &'tree Tree) -> Node<'tree> {
        self.roots
            .iter()
            .next()
            .copied()
            .map(|i| tree.at(i))
            .expect("non-empty infoset")
    }
}
