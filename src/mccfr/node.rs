use super::bucket::Bucket;
use super::path::Path;
use super::player::Player;
use crate::cards::street::Street;
use crate::clustering::encoding::Odds;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::play::action::Action;
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

/// TODO
/// very expensive operation to generate a new Bucket on the fly
/// every time we need a new Node. should find a way to
/// make cheap copies of Node.
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
        &self.data().bucket()
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

    /// expansion methods

    /// signature of the possible continuation edges emanating from this node after N-betting is cut off
    pub fn future(&self) -> Path {
        Path::from(self.continuations())
    }
    /// possible edges emanating from this node after N-betting is cut off
    pub fn continuations(&self) -> Vec<Edge> {
        let nraises = self
            .history()
            .iter()
            .rev()
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_aggro())
            .count();
        self.expand()
            .into_iter()
            .map(|(e, _)| e)
            .filter(|e| !e.is_raise() || nraises < crate::MAX_N_BETS)
            .collect::<Vec<Edge>>()
    }
    /// all actions available to the player at this node
    pub fn expand(&self) -> Vec<(Edge, Action)> {
        let mut options = self
            .data()
            .game()
            .legal()
            .into_iter()
            .map(|a| (Edge::from(a), a))
            .collect::<Vec<(Edge, Action)>>();
        if let Some(raise) = options.iter().position(|(_, a)| a.is_raise()) {
            if let Some(shove) = options.iter().position(|(_, a)| a.is_shove()) {
                if let Action::Raise(min) = options.get(raise).unwrap().1 {
                    if let Action::Shove(max) = options.get(shove).unwrap().1 {
                        options.remove(raise);
                        options.splice(
                            raise..raise,
                            self.raises()
                                .into_iter()
                                .map(|odds| (Edge::Raises(odds), Probability::from(odds)))
                                .map(|(e, p)| (e, p * self.data().game().pot() as Utility))
                                .map(|(e, x)| (e, x as Chips))
                                .filter(|(_, x)| min <= *x && *x < max)
                                .map(|(e, a)| (e, Action::Raise(a)))
                                .collect::<Vec<(Edge, Action)>>(),
                        );
                        return options;
                    }
                }
            }
        }
        options
    }
    /// discretized raise sizes, conditional on street and betting history
    pub fn raises(&self) -> Vec<Odds> {
        const PREF_RAISES: [Odds; 10] = [
            Odds(1, 4), // 0.25
            Odds(1, 3), // 0.33
            Odds(1, 2), // 0.50
            Odds(2, 3), // 0.66
            Odds(3, 4), // 0.75
            Odds(1, 1), // 1.00
            Odds(3, 2), // 1.50
            Odds(2, 1), // 2.00
            Odds(3, 1), // 3.00
            Odds(4, 1), // 4.00
        ];
        const FLOP_RAISES: [Odds; 5] = [
            Odds(1, 2), // 0.50
            Odds(3, 4), // 0.75
            Odds(1, 1), // 1.00
            Odds(3, 2), // 1.50
            Odds(2, 1), // 2.00
        ];
        const LATE_RAISES: [Odds; 2] = [
            Odds(1, 2), // 0.50
            Odds(1, 1), // 1.00
        ];
        const LAST_RAISES: [Odds; 1] = [
            Odds(1, 1), // 1.00
        ];
        match self.data().game().board().street() {
            Street::Pref => PREF_RAISES.to_vec(),
            Street::Flop => FLOP_RAISES.to_vec(),
            _ => match self
                .history()
                .iter()
                .rev()
                .take_while(|e| e.is_choice())
                .filter(|e| e.is_aggro())
                .count() // this is basically node.is_not_first_raise
            {
                0 => LATE_RAISES.to_vec(),
                _ => LAST_RAISES.to_vec(),
            },
        }
    }
}

impl std::fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "N{}", self.index().index())
    }
}
