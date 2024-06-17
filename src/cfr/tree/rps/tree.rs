use super::info::Info;
use super::node::Node;
use crate::cfr::tree::rps::action::Edge;
use crate::cfr::tree::rps::bucket::Bucket;
use crate::cfr::tree::rps::local::Child;
use crate::cfr::tree::rps::local::Local;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::ptr::NonNull;

/// trees
pub struct Tree {
    graph: Box<DiGraph<Node, Edge>>,
    infos: HashMap<Bucket, Info>,
    index: NodeIndex,
}

impl Tree {
    pub fn infosets(&self) -> Vec<&Info> {
        self.infos.values().collect()
    }
    pub fn new() -> Self {
        let mut this = Self {
            infos: HashMap::new(),
            graph: Box::new(DiGraph::new()),
            index: NodeIndex::new(0),
        };
        this.dfs(Self::root(), None);
        this
    }

    fn root() -> Local {
        Local::root()
    }
    fn children(&self, local: &Local) -> Vec<Child> {
        local.children()
    }

    fn bfs(&mut self) {
        let mut unexplored = vec![(Self::root(), None)];
        while let Some((next, edge)) = unexplored.pop() {
            let mut children = self.children(&next);
            let attached = self.attach_returnchildren(next, edge);
            while let Some(child) = children.pop() {
                unexplored.push((child.loca, Some(child.edge)));
            }
        }
    }

    fn dfs(&mut self, next: Local, edge: Option<Edge>) {
        // this method might stack overflow if tree is too deep. solution will be to use while loop since rust has no tail recursion optimization
        let children = self.children(&next);
        let attached = self.attach_returnchildren(next, edge);
        for child in children {
            let loca = child.loca;
            let edge = child.edge;
            self.index = attached;
            self.dfs(loca, Some(edge));
        }
    }
    fn attach_returnchildren(&mut self, loca: Local, edge: Option<Edge>) -> NodeIndex {
        let tail = NodeIndex::new(self.graph.node_count());
        match edge {
            Some(edge) => {
                self.attach_node(loca, tail);
                self.attach_edge(edge, tail);
                self.attach_info(tail);
            }
            None => {
                self.attach_node(loca, tail);
                self.attach_info(tail);
            }
        }
        tail
    }

    fn attach_node(&mut self, local: Local, index: NodeIndex) {
        self.graph.add_node(Node {
            local,
            index,
            graph: NonNull::from(self.graph.as_ref()),
        });
    }
    fn attach_edge(&mut self, edge: Edge, index: NodeIndex) {
        self.graph.add_edge(self.index, index, edge);
    }
    fn attach_info(&mut self, index: NodeIndex) {
        let attached = self.graph.node_weight(index).expect("valid node index");
        let bucket = *attached.bucket();
        let index = *attached.index();
        match self.infos.get_mut(&bucket) {
            None => {
                let roots = vec![index];
                let graph = NonNull::from(self.graph.as_ref());
                self.infos.insert(bucket, Info { roots, graph });
            }
            Some(info) => {
                info.roots.push(index);
            }
        }
    }
}
