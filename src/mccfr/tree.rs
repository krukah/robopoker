use super::bucket::Bucket;
use super::info::Info;
use crate::mccfr::edge::Edge;
use crate::mccfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::collections::BTreeMap;
use std::ptr::NonNull;

/// trees
pub struct Tree(Box<DiGraph<Node, Edge>>, BTreeMap<Bucket, Info>);

impl Tree {
    pub fn empty() -> Self {
        Self(Box::new(DiGraph::with_capacity(0, 0)), BTreeMap::new())
    }
    pub fn node(&self, head: NodeIndex) -> &Node {
        self.graph_ref()
            .node_weight(head)
            .expect("being spawned safely in recursion")
    }
    pub fn infosets(&self) -> Vec<Info> {
        self.1.values().cloned().collect()
    }
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
    pub fn graph_ptr(&self) -> NonNull<DiGraph<Node, Edge>> {
        NonNull::from(self.0.as_ref())
    }
    pub fn graph_ref(&self) -> &DiGraph<Node, Edge> {
        self.0.as_ref()
    }
    pub fn graph_mut(&mut self) -> &mut DiGraph<Node, Edge> {
        self.0.as_mut()
    }
}
