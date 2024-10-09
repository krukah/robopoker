use super::bucket::Bucket;
use super::info::Info;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::collections::BTreeMap;
use std::ptr::NonNull;

/// Represents the game tree structure used in Monte Carlo Counterfactual Regret Minimization (MCCFR).
///
/// The `Tree` struct contains two main components:
/// 1. A directed graph (`DiGraph`) representing the game tree, where nodes are game states and edges are actions.
/// 2. A mapping from `Bucket`s to `Info`sets, which groups similar game states together.
pub struct Tree(Box<DiGraph<Node, Edge>>, BTreeMap<Bucket, Info>);

impl Tree {
    /// Creates an empty game tree.
    ///
    /// This initializes a new `Tree` with an empty graph and an empty mapping of buckets to infosets.
    pub fn empty() -> Self {
        Self(Box::new(DiGraph::with_capacity(0, 0)), BTreeMap::new())
    }

    /// Retrieves a reference to a node in the game tree given its index.
    ///
    /// # Arguments
    ///
    /// * `head` - The index of the node to retrieve.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist in the graph.
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
    /// If the infoset for the node's bucket doesn't exist, it creates a new one.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to be added to an infoset.
    pub fn witness(&mut self, node: &Node) {
        let index = node.index();
        let bucket = node.bucket();
        if let Some(infoset) = self.1.get_mut(bucket) {
            infoset.add(index);
        } else {
            let graph = self.graph_ptr();
            let infoset = Info::from((index, graph));
            self.1.insert(bucket.clone(), infoset);
        }
    }

    /// Returns a non-null pointer to the underlying graph.
    pub fn graph_ptr(&self) -> NonNull<DiGraph<Node, Edge>> {
        NonNull::from(self.0.as_ref())
    }

    /// Returns a reference to the underlying graph.
    pub fn graph_ref(&self) -> &DiGraph<Node, Edge> {
        self.0.as_ref()
    }

    /// Returns a mutable reference to the underlying graph.
    pub fn graph_mut(&mut self) -> &mut DiGraph<Node, Edge> {
        self.0.as_mut()
    }
}
