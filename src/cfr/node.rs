use super::bucket::Bucket;
use super::player::Player;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::Utility;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::Direction::Incoming;
use petgraph::Direction::Outgoing;
use std::ptr::NonNull;

pub struct Node {
    graph: NonNull<DiGraph<Self, Edge>>,
    index: NodeIndex,
    datum: Data,
}

/// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
impl Node {
    // SAFETY: Node is only created by Tree...
    // who owns the Box<DiGraph>...
    // which ensures that the graph is valid...
    fn graph(&self) -> &DiGraph<Self, Edge> {
        unsafe { self.graph.as_ref() }
    }

    pub fn bucket(&self) -> &Bucket {
        self.datum.bucket()
    }
    pub fn player(&self) -> &Player {
        self.datum.player()
    }
    pub fn payoff(root: &Node, leaf: &Node) -> Utility {
        let stakes = leaf.datum.stakes();
        let direction = match root.player() {
            Player::P1 => 0. + 1.,
            Player::P2 => 0. - 1.,
            _ => unreachable!("payoff should not be queried for chance"),
        };
        direction * stakes
    }

    pub fn index(&self) -> NodeIndex {
        self.index
    }

    pub fn history(&self) -> Vec<&Edge> {
        match self.incoming() {
            None => vec![],
            Some(edge) => {
                let mut history = self.parent().expect("root handled above").history();
                history.push(edge);
                history
            }
        }
    }
    pub fn outgoing(&self) -> Vec<&Edge> {
        self.graph()
            .edges_directed(self.index, Outgoing)
            .map(|e| e.weight())
            .collect()
    }
    pub fn incoming(&self) -> Option<&Edge> {
        self.graph()
            .edges_directed(self.index, Incoming)
            .next()
            .map(|e| e.weight())
    }
    pub fn parent(&self) -> Option<&Self> {
        self.graph()
            .neighbors_directed(self.index, Incoming)
            .next()
            .map(|p| {
                self.graph()
                    .node_weight(p)
                    .expect("if incoming edge, then parent")
            })
    }
    pub fn children(&self) -> Vec<&Self> {
        self.graph()
            .neighbors_directed(self.index, Outgoing)
            .map(|c| {
                self.graph()
                    .node_weight(c)
                    .expect("if outgoing edge, then child")
            })
            .collect()
    }
    pub fn follow(&self, edge: &Edge) -> &Self {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .expect("valid edge to follow")
        //? TODO O(A) performance
    }
}

impl From<(NodeIndex, NonNull<DiGraph<Node, Edge>>, Data)> for Node {
    fn from((index, graph, datum): (NodeIndex, NonNull<DiGraph<Node, Edge>>, Data)) -> Self {
        Self {
            index,
            graph,
            datum,
        }
    }
}
