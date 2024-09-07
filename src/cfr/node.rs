use super::bucket::Bucket;
use super::player::Player;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::Utility;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::Direction::Incoming;
use petgraph::Direction::Outgoing;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Node {
    graph: Rc<RefCell<DiGraph<Self, Edge>>>,
    index: NodeIndex,
    datum: Data,
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
    pub fn player(&self) -> &Player {
        self.datum.player()
    }
    pub fn payoff(node: &Node, leaf: &Node) -> Utility {
        assert!(true, "should be terminal node");
        // todo!("use some Payoff::from(Showdown::from(Game)) type");
        let stakes = leaf.datum.stakes();
        let direction = match node.player() {
            Player::P1 => 0. + 1.,
            Player::P2 => 0. - 1.,
            _ => unreachable!("payoff should not be queried for chance"),
        };
        direction * stakes
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
    // SAFETY: Node is only created by Tree...
    // who owns the Box<DiGraph>...
    // which ensures that the graph is valid...
    fn graph(&self) -> &DiGraph<Self, Edge> {
        unsafe { self.graph.as_ptr().as_ref().expect("valid graph") }
    }
}

impl From<(NodeIndex, Rc<RefCell<DiGraph<Node, Edge>>>, Data)> for Node {
    fn from((index, graph, datum): (NodeIndex, Rc<RefCell<DiGraph<Node, Edge>>>, Data)) -> Self {
        Self {
            index,
            graph,
            datum,
        }
    }
}
