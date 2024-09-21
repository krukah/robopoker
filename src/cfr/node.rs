use super::bucket::Bucket;
use super::player::Player;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::play::continuation::Continuation;
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

impl From<(NodeIndex, NonNull<DiGraph<Node, Edge>>, Data)> for Node {
    fn from((index, graph, datum): (NodeIndex, NonNull<DiGraph<Node, Edge>>, Data)) -> Self {
        Self {
            index,
            graph,
            datum,
        }
    }
}

/// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
impl Node {
    pub fn datum(&self) -> &Data {
        &self.datum
    }
    pub fn index(&self) -> NodeIndex {
        self.index
    }

    pub fn bucket(&self) -> &Bucket {
        self.datum.bucket()
    }
    pub fn player(&self) -> Player {
        self.datum.player()
    }
    pub fn payoff(&self, player: &Player) -> Utility {
        let position = match player {
            Player::Choice(Continuation::Decision(x)) => x.to_owned(),
            _ => unreachable!("payoffs defined relative to decider"),
        };
        match player {
            Player::Choice(_) => unreachable!("payoffs defined relative to decider"),
            Player::Chance => self
                .datum()
                .game()
                .settlement()
                .get(position)
                .clone()
                .map(|settlement| settlement.pnl() as f32)
                .expect("player index in bounds"),
        }
    }

    /// Navigational methods
    ///
    /// maybe make these methods private and implement Walkable for Node?
    /// or add &Node as argument and impl Walkable for Tree?
    pub fn history(&self) -> Vec<&Edge> {
        if let (Some(edge), Some(head)) = (self.incoming(), self.parent()) {
            let mut history = head.history();
            history.push(edge);
            history
        } else {
            vec![]
        }
    }
    pub fn outgoing(&self) -> Vec<&Edge> {
        self.graph_ref()
            .edges_directed(self.index, Outgoing)
            .map(|e| e.weight())
            .collect()
    }
    pub fn incoming(&self) -> Option<&Edge> {
        self.graph_ref()
            .edges_directed(self.index, Incoming)
            .next()
            .map(|e| e.weight())
    }
    pub fn parent(&self) -> Option<&Self> {
        self.graph_ref()
            .neighbors_directed(self.index, Incoming)
            .next()
            .map(|p| {
                self.graph_ref()
                    .node_weight(p)
                    .expect("if incoming edge, then parent")
            })
    }
    pub fn children(&self) -> Vec<&Self> {
        self.graph_ref()
            .neighbors_directed(self.index, Outgoing)
            .map(|c| {
                self.graph_ref()
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
    pub fn leaves(&self) -> Vec<&Self> {
        if self.children().is_empty() {
            vec![self]
        } else {
            self.children()
                .iter()
                .flat_map(|child| child.leaves())
                .collect()
        }
    }
    /// SAFETY:
    /// we have logical assurance that lifetimes work out effectively:
    /// 'info: 'node: 'tree
    /// Info is created from a Node
    /// Node is created from a Tree
    /// Tree owns its Graph
    fn graph_ref(&self) -> &DiGraph<Self, Edge> {
        unsafe { self.graph.as_ref() }
    }
}
