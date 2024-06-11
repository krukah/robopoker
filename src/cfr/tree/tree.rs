use std::{collections::HashMap, ptr::NonNull};

use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};

use crate::cfr::traits::{action::E, bucket::B, local::L, player::C};

use super::{info::I, node::N};

/// trees
pub(crate) struct T {
    index: NodeIndex,
    graph: Box<DiGraph<N, E>>,
    infos: HashMap<B, I>,
}

impl T {
    pub fn infosets(&self) -> Vec<&I> {
        self.infos.values().collect()
    }
    pub fn new() -> Self {
        let root = L::root();
        let mut this = Self {
            infos: HashMap::new(),
            index: NodeIndex::new(0),
            graph: Box::new(DiGraph::new()),
        };
        this.insert(root);
        this.explore();
        this.bucketize();
        this
    }
    fn explore(&mut self) {
        while self.index.index() < self.graph.node_count() {
            for (child, edge) in self.spawn() {
                self.attach(child, edge);
            }
            self.index = NodeIndex::new(self.index.index() + 1);
        }
    }
    fn bucketize(&mut self) {
        for node in self
            .graph
            .node_weights()
            .filter(|n| *n.player() != C::Chance)
        {
            self.infos
                .entry(*node.bucket())
                .or_insert_with(|| I {
                    roots: Vec::new(),
                    graph: NonNull::from(&*self.graph),
                })
                .add(node);
        }
    }
    fn insert(&mut self, local: L) -> NodeIndex {
        let n = self.graph.add_node(N {
            local,
            graph: NonNull::from(&*self.graph),
            index: NodeIndex::new(self.graph.node_count()),
        });
        n
    }
    fn attach(&mut self, local: L, edge: E) -> EdgeIndex {
        let n = self.insert(local);
        let e = self.graph.add_edge(self.index, n, edge);
        e
    }
    fn spawn(&self) -> Vec<(L, E)> {
        self.graph
            .node_weight(self.index)
            .expect("self.point will be behind self.graph.node_count")
            .local()
            .spawn()
    }
}
