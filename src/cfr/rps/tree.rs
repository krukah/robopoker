use petgraph::graph::EdgeIndex;
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;

use crate::cfr::rps::action::RpsAction;
use crate::cfr::rps::bucket::RpsBucket;
use crate::cfr::rps::info::RpsInfo;
use crate::cfr::rps::node::RpsNode;
use crate::cfr::rps::player::RpsPlayer;
use crate::cfr::traits::tree::tree::Tree;
use std::collections::HashMap;
// use petgraph::graph::
use std::collections::HashSet;

pub(crate) struct RpsTree<'tree> {
    graph: Graph<RpsNode<'tree>, RpsAction>,
    infos: HashMap<RpsBucket, RpsInfo<'tree>>,
}

#[allow(unreachable_code, unused)]
impl<'t> RpsTree<'t> {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            infos: HashMap::new(),
        }
    }

    pub fn peek(&self, index: usize) -> &RpsNode<'t> {
        todo!("get node by something that RpsNode will have access to")
    }

    fn bfs(&mut self) {
        todo!("breadth-first build and recursive exploration for nodes")
    }
    fn spawns(&self, node: &RpsNode<'t>) -> Vec<RpsNode<'t>> {
        todo!("explore node continuations, incrementing local index for assignment")
    }
    fn attach(&mut self, node: RpsNode<'t>) {
        todo!("check node.bucket() and add to buckets")
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
