use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};

use super::node::RpsInner;
use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::bucket::RpsBucket;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::tree::tree::Tree;
use std::collections::HashMap;
use std::ptr::NonNull;

pub(crate) struct RpsTree<'t> {
    infos: HashMap<RpsBucket, RpsInfo<'t>>,
    graph: DiGraph<RpsNode, RpsAction>,
    index: NodeIndex,
}

impl<'t> RpsTree<'t> {
    pub fn new() -> Self {
        Self {
            infos: HashMap::new(),
            graph: DiGraph::new(),
            index: NodeIndex::new(0),
        }
    }
    pub fn expand(&mut self) {
        self.insert(Self::root());
        while self.index.index() < self.graph.node_count() {
            for (child, edge) in self
                .graph
                .node_weight(self.index)
                .expect("self.point will be behind self.graph.node_count")
                .inner()
                .spawn()
            {
                self.attach(child, edge);
            }
            self.advance();
        }
    }
    fn insert(&mut self, inner: RpsInner) -> NodeIndex {
        let index = NodeIndex::new(self.graph.node_count());
        let graph = NonNull::new(&mut self.graph).unwrap();
        self.graph.add_node(RpsNode::new(inner, index, graph))
    }
    fn attach(&mut self, node: RpsInner, edge: RpsAction) -> EdgeIndex {
        let child = self.insert(node);
        self.graph.add_edge(self.index, child, edge)
    }
    fn advance(&mut self) {
        self.index = NodeIndex::new(self.index.index() + 1);
    }
    fn root() -> RpsInner {
        RpsInner
    }
}

impl<'t> Tree for RpsTree<'t> {
    fn infos(&self) -> Vec<&RpsInfo<'t>> {
        self.infos.values().collect()
    }

    type TPlayer = RpsPlayer;
    type TEdge = RpsAction;
    type TNode = RpsNode;
    type TInfo = RpsInfo<'t>;
}
