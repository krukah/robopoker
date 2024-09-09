use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::node::Node;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::collections::BTreeMap;
use std::ptr::NonNull;

/// trees
pub struct Tree {
    graph: Box<DiGraph<Node, Edge>>,
    infos: BTreeMap<Bucket, Info>,
}

impl Tree {
    pub fn empty() -> Self {
        let infos = BTreeMap::new();
        let graph = Box::new(DiGraph::with_capacity(0, 0));
        Self { infos, graph }
    }
    pub fn infosets(&self) -> Vec<Info> {
        self.infos.values().cloned().collect()
    }
    pub fn witness(&mut self, node: &Node) {
        let index = node.index();
        let bucket = node.bucket();
        if let Some(infoset) = self.infos.get_mut(bucket) {
            infoset.add(index);
        } else {
            let graph = self.graph_raw();
            let infoset = Info::from((index, graph));
            self.infos.insert(bucket.clone(), infoset);
        }
    }

    pub fn node(&self, head: NodeIndex) -> &Node {
        self.graph_ref()
            .node_weight(head)
            .expect("being spawned safely in recursion")
    }
    pub fn graph_raw(&self) -> NonNull<DiGraph<Node, Edge>> {
        NonNull::from(self.graph.as_ref())
    }
    pub fn graph_ref(&self) -> &DiGraph<Node, Edge> {
        self.graph.as_ref()
    }
    pub fn graph_mut(&mut self) -> &mut DiGraph<Node, Edge> {
        self.graph.as_mut()
    }
}
