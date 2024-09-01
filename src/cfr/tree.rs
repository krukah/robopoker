use crate::cfr::bucket::Bucket;
use crate::cfr::data::Child;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::cfr::info::Info;
use crate::cfr::node::Node;
use crate::cfr::player::Player;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::ptr::NonNull;

/// trees
pub struct Tree {
    graph: Box<DiGraph<Node, Edge>>,
    infos: HashMap<Bucket, Info>,
}

impl Tree {
    pub fn blocks(&self) -> Vec<&Info> {
        self.infos.values().collect()
    }
    pub fn new() -> Self {
        let mut this = Self {
            infos: HashMap::new(),
            graph: Box::new(DiGraph::new()),
        };
        this.dfs();
        this.bucketize();
        this
    }

    fn dfs(&mut self) {
        let root = (Self::root(), None, NodeIndex::from(0));
        let mut parents = vec![root];
        while let Some(parent) = parents.pop() {
            let mut children = self.spawn(&parent.0);
            let (data, from, head) = parent;
            let node = self.engulf(data); // , index
            let tail = self.attach(node, from, head); // , mut index
            while let Some(child) = children.pop() {
                let data = child.data;
                let edge = Some(child.edge);
                parents.push((data, edge, tail));
            }
        }
    }

    fn bucketize(&mut self) {
        for node in self.graph.node_weights() {
            let index = node.index();
            let player = node.player();
            let bucket = node.bucket();
            if player == &Player::Chance {
                continue;
            } else {
                match self.infos.get_mut(bucket) {
                    Some(info) => info.push(index),
                    None => {
                        let info = Info::from((index, self.graph()));
                        let bucket = bucket.to_owned();
                        self.infos.insert(bucket, info);
                    }
                }
            }
        }
    }

    fn root() -> Data {
        Data::root()
    }
    fn spawn(&self, data: &Data) -> Vec<Child> {
        data.spawn()
    }
    fn index(&self) -> NodeIndex {
        NodeIndex::new(self.graph.node_count())
    }
    fn graph(&self) -> NonNull<DiGraph<Node, Edge>> {
        NonNull::from(self.graph.as_ref())
    }
    fn engulf(&self, data: Data) -> Node {
        Node::from((self.index(), self.graph(), data))
    }
    fn attach(&mut self, node: Node, edge: Option<Edge>, head: NodeIndex) -> NodeIndex {
        let tail = self.index();
        if let Some(edge) = edge {
            self.graph.add_node(node);
            self.graph.add_edge(head, tail, edge);
        } else {
            self.graph.add_node(node);
        }
        tail
    }
}
