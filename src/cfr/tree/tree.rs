use super::info::Info;
use super::node::Node;
use crate::cfr::traits::action::Edge;
use crate::cfr::traits::bucket::Bucket;
use crate::cfr::traits::local::Local;
use crate::cfr::traits::player::Player;
use petgraph::graph::DiGraph;
use petgraph::graph::EdgeIndex;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::ptr::NonNull;

/// trees
pub(crate) struct Tree {
    index: NodeIndex,
    graph: Box<DiGraph<Node, Edge>>,
    infos: HashMap<Bucket, Info>,
}

impl Tree {
    pub fn infosets(&self) -> Vec<&Info> {
        self.infos.values().collect()
    }
    pub fn new() -> Self {
        let root = Local::root();
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
            .filter(|n| *n.player() != Player::Chance)
        {
            self.infos
                .entry(*node.bucket())
                .or_insert_with(|| Info {
                    roots: Vec::new(),
                    graph: NonNull::from(&*self.graph),
                })
                .add(node);
        }
    }
    fn insert(&mut self, local: Local) -> NodeIndex {
        let n = self.graph.add_node(Node {
            local,
            graph: NonNull::from(&*self.graph),
            index: NodeIndex::new(self.graph.node_count()),
        });
        n
    }
    fn attach(&mut self, local: Local, edge: Edge) -> EdgeIndex {
        let n = self.insert(local);
        let e = self.graph.add_edge(self.index, n, edge);
        e
    }
    fn spawn(&self) -> Vec<(Local, Edge)> {
        self.graph
            .node_weight(self.index)
            .expect("self.point will be behind self.graph.node_count")
            .local()
            .spawn()
    }
}
