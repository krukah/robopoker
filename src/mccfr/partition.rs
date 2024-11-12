use super::bucket::Bucket;
use super::tree::Tree;
use crate::mccfr::info::Info;
use crate::mccfr::node::Node;
use petgraph::graph::NodeIndex;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Partition(BTreeMap<Bucket, Vec<NodeIndex>>);

impl Partition {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn infos(&self, tree: Arc<Tree>) -> Vec<Info> {
        self.0
            .iter()
            .map(|(_, indices)| Info::from((tree.clone(), indices.clone())))
            .collect()
    }
    pub fn witness(&mut self, node: &Node) {
        self.0
            .entry(node.bucket())
            .or_insert_with(Vec::new)
            .push(node.index());
    }
}
