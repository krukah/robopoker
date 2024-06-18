use super::info::Info;
use super::node::Node;
use crate::cfr::tree::rps::action::Edge;
use crate::cfr::tree::rps::bucket::Bucket;
use crate::cfr::tree::rps::local::Child;
use crate::cfr::tree::rps::local::Data;
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
    pub fn infosets(&self) -> Vec<&Info> {
        self.infos.values().collect()
    }
    pub fn new() -> Self {
        let mut this = Self {
            infos: HashMap::new(),
            graph: Box::new(DiGraph::new()),
        };
        this.bfs();
        this
    }

    fn root() -> Data {
        Data::root()
    }
    fn children(&self, data: &Data) -> Vec<Child> {
        data.children()
    }
    fn bfs(&mut self) {
        let root = (Self::root(), None, NodeIndex::from(0));
        let mut descendants = vec![root];
        while let Some(descendant) = descendants.pop() {
            let data = descendant.0;
            let from = descendant.1;
            let head = descendant.2;
            let mut children = self.children(&data);
            let this = self.attach(data, from, head);
            while let Some(child) = children.pop() {
                let data = child.data;
                let from = Some(child.edge);
                descendants.push((data, from, this));
            }
        }
    }

    fn attach(&mut self, data: Data, from: Option<Edge>, head: NodeIndex) -> NodeIndex {
        let next = self.next();
        let node = self.wrap(data);
        let bucket = node.bucket();
        // (Bucket, NodeIndex) -> ()
        // add nodeIndex to infoset before inserting ownership into graph.
        // may want to factor this out to allow for custom infoset iteration logic, such as skipping non-traversers or chance nodes
        if let Some(info) = self.infos.get_mut(bucket) {
            info.roots.push(next);
        } else {
            let mut info = Info {
                roots: Vec::new(),
                graph: self.graph(),
            };
            info.roots.push(next);
            self.infos.insert(*bucket, info);
        }
        // (Node, Option<Edge>) -> ()
        // add node to graph, giving ownership
        // next index is calculated before insertion to avoid off-by-one errors
        if let Some(e) = from {
            self.graph.add_node(node);
            self.graph.add_edge(head, next, e);
        } else {
            self.graph.add_node(node);
        }
        // after all of this, increment the index
        next
    }

    fn wrap(&self, data: Data) -> Node {
        Node {
            data,
            index: self.next(),
            graph: self.graph(),
        }
    }
    fn next(&self) -> NodeIndex {
        NodeIndex::new(self.graph.node_count())
    }
    fn graph(&self) -> NonNull<DiGraph<Node, Edge>> {
        NonNull::from(self.graph.as_ref())
    }
}
