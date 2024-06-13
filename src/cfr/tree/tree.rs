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
    next: NodeIndex,
    last: NodeIndex,
    edge: EdgeIndex,
    graph: Box<DiGraph<Node, Edge>>,
    infos: HashMap<Bucket, Info>,
}

impl Tree {
    pub fn infosets(&self) -> Vec<&Info> {
        self.infos.values().collect()
    }

    // allocation of the tree from scratch
    pub fn new() -> Self {
        let mut this = Self {
            next: NodeIndex::new(0),
            last: NodeIndex::new(0),
            edge: EdgeIndex::new(0),
            infos: HashMap::new(),
            graph: Box::new(DiGraph::new()),
        };
        this.seed();
        this.grow();
        this.bucketize();
        this
    }

    fn seed(&mut self) -> NodeIndex {
        let n = self.graph.add_node(self.new_node(Self::root()));
        n
    }
    fn grow(&mut self) {
        // move to next unexplored node in the BFS
        while self.next.index() < self.graph.node_count() {
            for (child, edge) in self.spawn() {
                self.last = self.graph.add_node(self.new_node(child));
                self.edge = self.graph.add_edge(self.next, self.last, edge);
            }
            self.next = NodeIndex::new(self.next.index() + 1);
        }
    }
    fn bucketize(&mut self) {
        for node in self
            .graph
            .node_weights()
            .filter(|n| *n.player() != Player::Chance)
        {
            match self.infos.get_mut(node.bucket()) {
                Some(info) => {
                    info.roots.push(*node.index());
                }
                None => {
                    let info = self.new_info(node);
                    let bucket = node.bucket().clone();
                    self.infos.insert(bucket, info);
                }
            }
        }
    }

    // allocation of new inner values of nodes, either from scratch (root)   or parent (spawn)
    // these may or may not be bound to Inner/Local rather than Tree
    // for now we can generate them locally from Inner, but we may want tree-level context
    fn root() -> Local {
        Local::root()
    }
    fn spawn(&self) -> Vec<(Local, Edge)> {
        self.graph
            .node_weight(self.next)
            .expect("self.next (unexplored) will be behind self.graph.node_count")
            .local()
            .spawn()
    }

    // allocation of our wrapper types, Node and Info, that are helpful for implementing traversal in the CFR algorithm
    fn new_node(&self, local: Local) -> Node {
        Node {
            local,
            graph: NonNull::from(self.graph.as_ref()),
            index: NodeIndex::new(self.graph.node_count()),
        }
    }
    fn new_info(&self, node: &Node) -> Info {
        Info {
            roots: vec![*node.index()],
            graph: NonNull::from(self.graph.as_ref()),
        }
    }
}
