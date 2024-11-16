use super::bucket::Bucket;
use super::path::Path;
use super::player::Player;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::play::ply::Ply;
use crate::Chips;
use crate::Probability;
use crate::Utility;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::Direction::Incoming;
use petgraph::Direction::Outgoing;

/// A Node is a wrapper around a NodeIndex and a &Graph.
/// because they are thin wrappers around an index, they're
/// cheap to Copy. holding reference to Graph is useful
/// for navigational methods.
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

impl<'tree> Node<'tree> {
    pub fn spawn(&self, index: NodeIndex) -> Node<'tree> {
        Self::from((index, self.graph()))
    }
    pub fn data(&self) -> &Data {
        &self
            .graph
            .node_weight(self.index())
            .expect("valid node index")
    }
    pub fn bucket(&self) -> &Bucket {
        self.data().bucket()
    }
    pub fn index(&self) -> NodeIndex {
        self.index
    }
    pub fn player(&self) -> Player {
        self.data().player()
    }
    pub fn payoff(&self, player: &Player) -> Utility {
        match player {
            Player(Ply::Terminal) => unreachable!(),
            Player(Ply::Chance) => unreachable!(),
            Player(Ply::Choice(x)) => self
                .data()
                .game()
                .settlements()
                .get(*x)
                .map(|settlement| settlement.pnl() as f32)
                .expect("player index in bounds"),
        }
    }

    /// Navigational methods
    pub fn futures(&self, edge: &Edge) -> Vec<Edge> {
        self.history()
            .into_iter()
            .chain(std::iter::once(edge))
            .rev()
            .take_while(|e| e.is_choice())
            .cloned()
            .collect()
    }
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
    pub fn follow(&self, edge: &Edge) -> Option<Node<'tree>> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.spawn(child.index()))
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
    pub fn graph(&self) -> &'tree DiGraph<Data, Edge> {
        self.graph
    }

    pub fn localization(&self) -> Bucket {
        let present = self.data().abstraction().clone();
        let subgame = Path::from(self.subgame()); // could be from &'tree [Edge]
        let choices = Path::from(self.choices()); // could be from &'tree [Edge]
        Bucket::from((subgame, present, choices))
    }
}

use super::odds::Odds;
use crate::cards::street::Street;
use crate::play::action::Action;

impl Node<'_> {
    /// convert an Edge into an Action by using Game state to
    /// determine free parameters (stack size, pot size, etc)
    pub fn action(&self, edge: &Edge) -> Action {
        let game = self.data().game();
        match &edge {
            Edge::Raise(o) => Action::Raise((game.pot() as Utility * Utility::from(*o)) as Chips),
            Edge::Shove => Action::Shove(game.to_shove()),
            Edge::Call => Action::Call(game.to_call()),
            Edge::Draw => Action::Draw(game.draw()),
            Edge::Fold => Action::Fold,
            Edge::Check => Action::Check,
        }
    }
    /// returns the set of all possible actions from the current node
    /// this is useful for generating a set of children for a given node
    /// broadly goes from Node -> Game -> Action -> Edge
    pub fn choices(&self) -> Vec<Edge> {
        self.data()
            .game()
            .legal()
            .into_iter()
            .flat_map(|a| self.generalize(a))
            .collect()
    }
    /// returns the subgame history of the current node
    /// within the same Street of action.
    /// this should be made lazily in the future
    pub fn subgame(&self) -> Vec<Edge> {
        self.history()
            .into_iter()
            .take_while(|e| e.is_choice())
            .copied()
            .collect()
    }
    /// returns a set of possible raises given the current history
    /// we truncate in a few cases:
    /// - prevent N-betting explosion of raises
    /// - allow for finer-grained exploration in early streets
    /// - on the last street, restrict raise amounts so smaller grid
    fn raises(&self) -> Vec<Odds> {
        let n = self.subgame().len();
        if n > crate::MAX_N_BETS {
            vec![]
        } else {
            match self.data().game().board().street() {
                Street::Pref => Odds::PREF_RAISES.to_vec(),
                Street::Flop => Odds::FLOP_RAISES.to_vec(),
                _ => match n {
                    0 => Odds::LATE_RAISES.to_vec(),
                    _ => Odds::LAST_RAISES.to_vec(),
                },
            }
        }
    }
    /// generalization of mapping a concrete Action into a set of abstract Vec<Edge>
    /// this is mostly useful for enumerating a set of desired Raises
    /// which can be generated however.
    /// the contract is that the Actions returned by Game are legal,
    /// but the Raise amount can take any value >= the minimum provided by Game.
    fn generalize(&self, action: Action) -> Vec<Edge> {
        if let Action::Raise(_) = action {
            let min = self.data().game().to_raise();
            let max = self.data().game().to_shove() - 1;
            self.raises()
                .into_iter()
                .map(|o| (o, Probability::from(o)))
                .map(|(o, p)| (o, p * self.data().game().pot() as Utility))
                .map(|(o, x)| (o, x as Chips))
                .filter(|(_, x)| min <= *x && *x <= max)
                .map(|(o, _)| Edge::from(o))
                .collect()
        } else {
            vec![Edge::from(action)]
        }
    }
}

impl std::fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "N{}", self.index().index())
    }
}
