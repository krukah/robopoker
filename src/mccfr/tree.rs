use super::data::Data;
use super::info::Info;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::EdgeIndex;
use petgraph::graph::NodeIndex;

/// Represents the game tree structure used in Monte Carlo Counterfactual Regret Minimization (MCCFR).
///
/// The `Tree` struct contains two main components:
/// 1. A directed graph (`DiGraph`) representing the game tree, where nodes are game states and edges are actions.
/// 2. A mapping from `Bucket`s to `Info`sets, which groups similar game states together.
pub struct Tree(DiGraph<Data, Edge>);

impl Tree {
    pub fn at(&self, index: NodeIndex) -> Node {
        Node::from((index, &self.0))
    }
    pub fn empty() -> Self {
        Self(DiGraph::with_capacity(0, 0))
    }
    pub fn graph(&self) -> &DiGraph<Data, Edge> {
        &self.0
    }
    pub fn insert(&mut self, spot: Data) -> NodeIndex {
        self.0.add_node(spot)
    }
    pub fn attach(&mut self, edge: Edge, head: NodeIndex, tail: NodeIndex) -> EdgeIndex {
        self.0.add_edge(head, tail, edge)
    }
}

impl Iterator for Tree {
    type Item = Info;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
