use super::bucket::Bucket;
use super::player::Player;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::play::transition::Transition;
use crate::Utility;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::Direction::Incoming;
use petgraph::Direction::Outgoing;

#[derive(Debug, Clone, Copy)]
pub struct Node<'tree> {
    index: NodeIndex,
    graph: &'tree DiGraph<Data, Edge>,
}

impl<'tree> From<(NodeIndex, &'tree DiGraph<Data, Edge>)> for Node<'tree> {
    fn from((index, graph): (NodeIndex, &'tree DiGraph<Data, Edge>)) -> Self {
        Self { index, graph }
    }
}

/// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
impl<'tree> Node<'tree> {
    pub fn spot(&self) -> &Data {
        &self
            .graph
            .node_weight(self.index())
            .expect("valid node index")
    }
    pub fn index(&self) -> NodeIndex {
        self.index
    }
    pub fn bucket(&self) -> &Bucket {
        self.spot().bucket()
    }
    pub fn player(&self) -> Player {
        self.spot().player()
    }
    pub fn payoff(&self, player: &Player) -> Utility {
        let position = match player {
            Player::Choice(Transition::Choice(x)) => x.clone(),
            _ => unreachable!("payoffs defined relative to decider"),
        };
        match player {
            Player::Choice(_) => unreachable!("payoffs defined relative to decider"),
            Player::Chance => self
                .spot()
                .game()
                .settlement()
                .get(position)
                .map(|settlement| settlement.pnl() as f32)
                .expect("player index in bounds"),
        }
    }

    /// Navigational methods
    ///
    /// maybe make these methods private and implement Walkable for Node?
    /// or add &Node as argument and impl Walkable for Tree?
    pub fn history(&self) -> Vec<&'tree Edge> {
        if let (Some(edge), Some(head)) = (self.incoming(), self.parent()) {
            let mut history = head.history();
            history.push(edge);
            history
        } else {
            vec![]
        }
    }
    pub fn outgoing(&self) -> Vec<&'tree Edge> {
        self.graph()
            .edges_directed(self.index(), Outgoing)
            .map(|edge| edge.weight())
            .collect()
    }
    pub fn incoming(&self) -> Option<&'tree Edge> {
        self.graph()
            .edges_directed(self.index(), Incoming)
            .next()
            .map(|edge| edge.weight())
    }
    pub fn parent(&self) -> Option<Node<'tree>> {
        self.graph()
            .neighbors_directed(self.index(), Incoming)
            .next()
            .map(|index| self.spawn(index))
    }
    pub fn children(&self) -> Vec<Node<'tree>> {
        self.graph()
            .neighbors_directed(self.index(), Outgoing)
            .map(|index| self.spawn(index))
            .collect()
    }
    pub fn follow(&self, edge: &Edge) -> Node<'tree> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.spawn(child.index()))
            .expect("valid edge to follow")
    }
    pub fn leaves(&self) -> Vec<Node<'tree>> {
        if self.children().is_empty() {
            vec![self.clone()]
        } else {
            self.children()
                .iter()
                .flat_map(|child| child.leaves())
                .collect()
        }
    }
    fn spawn(&self, index: NodeIndex) -> Node<'tree> {
        Self::from((index, self.graph()))
    }
    /// SAFETY:
    /// we have logical assurance that lifetimes work out effectively:
    /// 'info: 'node: 'tree
    /// Info is created from a Node
    /// Node is created from a Tree
    /// Tree owns its Graph
    pub fn graph(&self) -> &'tree DiGraph<Data, Edge> {
        self.graph
    }
}
