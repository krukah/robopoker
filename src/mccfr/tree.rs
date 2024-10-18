use super::bucket::Bucket;
use super::info::Info;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::EdgeIndex;
use petgraph::graph::NodeIndex;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Represents the game tree structure used in Monte Carlo Counterfactual Regret Minimization (MCCFR).
///
/// The `Tree` struct contains two main components:
/// 1. A directed graph (`DiGraph`) representing the game tree, where nodes are game states and edges are actions.
/// 2. A mapping from `Bucket`s to `Info`sets, which groups similar game states together.
pub struct Tree(Arc<DiGraph<Node, Edge>>, BTreeMap<Bucket, Info>);

impl Tree {
    /// Creates an empty game tree.
    ///
    /// This initializes a new `Tree` with an empty graph and an empty mapping of buckets to infosets.
    pub fn empty() -> Self {
        Self(Arc::new(DiGraph::with_capacity(0, 0)), BTreeMap::new())
    }

    /// Retrieves a reference to a node in the game tree given its index.
    ///
    pub fn node(&self, head: NodeIndex) -> &Node {
        self.graph_ref()
            .node_weight(head)
            .expect("being spawned safely in recursion")
    }

    /// Returns a vector of all infosets in the game tree.
    pub fn infosets(&self) -> Vec<Info> {
        self.1.values().cloned().collect()
    }

    /// Adds a node to its corresponding infoset in the game tree.
    ///
    pub fn witness(&mut self, node: &Node) {
        let index = node.index();
        let bucket = node.bucket();
        if let Some(infoset) = self.1.get_mut(bucket) {
            infoset.add(index);
        } else {
            let graph = self.graph_arc();
            let infoset = Info::from((index, graph));
            self.1.insert(bucket.clone(), infoset);
        }
    }

    /// Returns a non-null pointer to the underlying graph.
    pub fn graph_arc(&self) -> Arc<DiGraph<Node, Edge>> {
        self.0.clone()
    }

    /// Returns a reference to the underlying graph.
    pub fn graph_ref(&self) -> &DiGraph<Node, Edge> {
        self.0.as_ref()
    }
    pub fn add_node(&mut self, _node: Node) -> NodeIndex {
        todo!()
    }

    pub fn add_edge(&mut self, _root: NodeIndex, _head: NodeIndex, _edge: Edge) -> EdgeIndex {
        todo!()
    }
}
