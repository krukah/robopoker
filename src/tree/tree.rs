use super::info::Info;
use super::node::Node;
use super::player::Player;
use crate::tree::action::Edge;
use crate::tree::bucket::Bucket;
use crate::tree::data::Child;
use crate::tree::data::Data;
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

    fn index(&self) -> NodeIndex {
        NodeIndex::new(self.graph.node_count())
    }
    fn graph(&self) -> NonNull<DiGraph<Node, Edge>> {
        NonNull::from(self.graph.as_ref())
    }
    fn wrap(&self, data: Data) -> Node {
        Node {
            data,
            index: self.index(),
            graph: self.graph(),
        }
    }

    fn dfs(&mut self) {
        // let index = 0;
        let root = (Self::root(), None, NodeIndex::from(0));
        let mut parents = vec![root];
        while let Some(parent) = parents.pop() {
            let mut children = self.spawn(&parent.0);
            let data = parent.0;
            let from = parent.1;
            let head = parent.2;
            let node = self.wrap(data); // , index
            let tail = self.attach(node, from, head); // , mut index
            while let Some(child) = children.pop() {
                let data = child.data;
                let from = Some(child.edge);
                parents.push((data, from, tail));
            }
        }
    }

    fn bucketize(&mut self) {
        for node in self.graph.node_weights() {
            if node.player() == &Player::Chance {
                continue;
            } else if let Some(info) = self.infos.get_mut(node.bucket()) {
                info.roots.push(node.index);
            } else {
                let mut info = Info {
                    roots: Vec::new(),
                    graph: self.graph(),
                };
                info.roots.push(node.index);
                self.infos.insert(*node.bucket(), info);
            }
        }
    }

    fn attach(&mut self, node: Node, from: Option<Edge>, head: NodeIndex) -> NodeIndex {
        let tail = self.index();
        if let Some(edge) = from {
            self.graph.add_node(node);
            self.graph.add_edge(head, tail, edge);
        } else {
            self.graph.add_node(node);
        }
        tail
    }

    // tree-building methods.
    // memory-allocating.
    // full tree defined recursively by ::root() + ::children()

    fn root() -> Data {
        // MARK: very different
        Data(0)
    }
    fn spawn(&self, data: &Data) -> Vec<Child> {
        // MARK: very different
        match data.0 {
            // P1 moves
            00 => vec![
                Child {
                    data: Data(01),
                    edge: Edge::RO,
                },
                Child {
                    data: Data(02),
                    edge: Edge::PA,
                },
                Child {
                    data: Data(03),
                    edge: Edge::SC,
                },
            ],
            // P2 moves
            01 => vec![
                Child {
                    data: Data(04),
                    edge: Edge::RO,
                },
                Child {
                    data: Data(05),
                    edge: Edge::PA,
                },
                Child {
                    data: Data(06),
                    edge: Edge::SC,
                },
            ],
            02 => vec![
                Child {
                    data: Data(07),
                    edge: Edge::RO,
                },
                Child {
                    data: Data(08),
                    edge: Edge::PA,
                },
                Child {
                    data: Data(09),
                    edge: Edge::SC,
                },
            ],
            03 => vec![
                Child {
                    data: Data(10),
                    edge: Edge::RO,
                },
                Child {
                    data: Data(11),
                    edge: Edge::PA,
                },
                Child {
                    data: Data(12),
                    edge: Edge::SC,
                },
            ],
            // terminal nodes
            04..=12 => Vec::new(),
            //
            _ => unreachable!(),
        }
    }
}
