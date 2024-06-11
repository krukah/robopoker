use crate::cfr::traits::action::Edge;
use crate::cfr::traits::bucket::Bucket;
use crate::cfr::traits::local::Local;
use crate::cfr::traits::player::Player;
use crate::Utility;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::Direction::Incoming;
use petgraph::Direction::Outgoing;
use std::ptr::NonNull;

pub(crate) struct Node {
    pub graph: NonNull<DiGraph<Self, Edge>>,
    pub index: NodeIndex,
    pub local: Local,
}

/// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
impl Node {
    // observability
    pub fn local(&self) -> &Local {
        &self.local
    }
    pub fn index(&self) -> &NodeIndex {
        &self.index
    }
    pub fn bucket(&self) -> &Bucket {
        self.local().bucket()
    }
    pub fn player(&self) -> &Player {
        self.local().player()
    }
    pub fn payoff(&self, player: &Player) -> Utility {
        self.local().payoff(player)
    }
    // walkability
    pub fn incoming(&self) -> Option<&Edge> {
        self.graph()
            .edges_directed(*self.index(), Incoming)
            .next()
            .map(|e| e.weight())
    }
    pub fn outgoing(&self) -> Vec<&Edge> {
        self.graph()
            .edges_directed(*self.index(), Outgoing)
            .map(|e| e.weight())
            .collect()
    }
    pub fn parent<'tree>(&'tree self) -> Option<&'tree Self> {
        self.graph()
            .neighbors_directed(*self.index(), Incoming)
            .next()
            .map(|p| {
                self.graph()
                    .node_weight(p)
                    .expect("tree property: if incoming edge, then parent")
            })
    }
    pub fn children<'tree>(&'tree self) -> Vec<&'tree Self> {
        self.graph()
            .neighbors_directed(*self.index(), Outgoing)
            .map(|c| {
                self.graph()
                    .node_weight(c)
                    .expect("tree property: if outgoing edge, then child")
            })
            .collect()
    }
    pub fn descendants<'tree>(&'tree self) -> Vec<&'tree Self> {
        match self.children().len() {
            0 => vec![&self],
            _ => self
                .children()
                .iter()
                .map(|child| child.descendants())
                .flatten()
                .collect(),
        }
    }
    pub fn follow<'tree>(&'tree self, edge: &Edge) -> &'tree Self {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .unwrap()
        //? TODO O(A) performance
    }
    // SAFETY: Node is only created by Tree...
    // who owns the Box<DiGraph>...
    // which ensures that the graph is valid...
    fn graph(&self) -> &DiGraph<Self, Edge> {
        unsafe { self.graph.as_ref() }
    }
}
