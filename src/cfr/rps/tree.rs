use petgraph::graph::DiGraph;
use petgraph::graph::EdgeIndex;
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Incoming;
use petgraph::Outgoing;

use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::bucket::RpsBucket;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::tree::tree::Tree;
use std::collections::HashMap;

pub(crate) struct RpsTree<'tree> {
    // nodes is a link between graph and nodes that effectively allows for circular reference via indexing
    graph: Graph<RpsNode<'tree>, RpsAction>,
    infos: HashMap<RpsBucket, RpsInfo<'tree>>,
}

#[allow(unreachable_code, unused)]
impl<'t> RpsTree<'t> {
    fn root() -> RpsNode<'t> {
        todo!("build/define root node for game tree")
    }

    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            infos: HashMap::new(),
        }
    }
    pub fn graph(&self) -> &Graph<RpsNode<'t>, RpsAction> {
        &self.graph
    }

    pub fn children(&self, idx: NodeIndex) -> Vec<&RpsNode<'t>> {
        self.graph
            .edges_directed(idx, Outgoing)
            .map(|e| e.target())
            .map(|i| {
                self.graph
                    .node_weight(i)
                    .expect("follwed directed edge downward in tree to node")
            })
            .collect()
    }
    fn spawns(&self, index: NodeIndex) -> Vec<RpsNode<'t>> {
        todo!("explore node continuations, incrementing local index for assignment")
    }
    fn attach(&mut self, node: RpsNode<'t>) {
        todo!("check node.bucket() and add to buckets")
    }
    fn bfs(&mut self) {
        todo!("breadth-first build and recursive exploration for nodes")
    }
}

impl<'t> Tree for RpsTree<'t> {
    fn infos(&self) -> Vec<&RpsInfo<'t>> {
        self.infos.values().collect()
    }

    type TPlayer = RpsPlayer;
    type TEdge = RpsAction;
    type TNode = RpsNode<'t>;
    type TInfo = RpsInfo<'t>;
}
