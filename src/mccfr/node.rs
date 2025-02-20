use super::bucket::Bucket;
use super::odds::Odds;
use super::path::Path;
use super::player::Player;
use crate::cards::street::Street;
use crate::gameplay::action::Action;
use crate::gameplay::game::Game;
use crate::gameplay::ply::Turn;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
use crate::Chips;
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
            Player(Turn::Terminal) | Player(Turn::Chance) => unreachable!(),
            Player(Turn::Choice(x)) => self
                .data()
                .game()
                .settlements()
                .get(*x)
                .map(|settlement| settlement.pnl() as f32)
                .expect("player index in bounds"),
        }
    }

    /// navigation methods

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

    /// this is the bucket that we use to lookup the Policy
    /// it is the product of the present abstraction, the
    /// history of choices leading up to the node, and the
    /// set of available continuations from the node.
    pub fn realize(&self) -> Bucket {
        let present = self.data().abstraction().clone();
        let history = Path::from(self.recall()); //              TODO: zero copy
        let choices = Path::from(self.calculate_continuations()); // TODO: zero copy
        Bucket::from((history, present, choices))
    }

    /// determine the set of branches that could be taken from this node
    /// this determines what Bucket we end up in since Tree::attach()
    /// uses this to assign Buckets to Data upon insertion
    pub fn branches(&self) -> Vec<(Edge, Game)> {
        self.reference_continuations()
            .into_iter()
            .map(|e| (e, self.actionize(&e)))
            .map(|(e, a)| (e.clone(), self.data().game().apply(a)))
            .collect()
    }
    /// lookup precomputed continuations from the node data
    fn reference_continuations(&self) -> Vec<Edge> {
        Vec::<Edge>::from(self.data().bucket().2.clone())
    }
    /// returns the set of all possible actions from the current node
    /// this is useful for generating a set of children for a given node
    /// broadly goes from Node -> Game -> Action -> Edge
    fn calculate_continuations(&self) -> Vec<Edge> {
        self.data()
            .game()
            .legal()
            .into_iter()
            .flat_map(|a| self.edgeifies(a))
            .collect()
    }
    /// convert an Edge into an Action by using Game state to
    /// determine free parameters (stack size, pot size, etc)
    ///
    /// NOTE
    /// this conversion is not injective, as multiple edges may
    /// represent the same action. moreover, we "snap" raises to be
    /// within range of legal bet sizes, so sometimes Raise(5:1) yields
    /// an identical Game node as Raise(1:1) or Shove.
    fn actionize(&self, edge: &Edge) -> Action {
        let game = self.data().game();
        match &edge {
            Edge::Check => Action::Check,
            Edge::Fold => Action::Fold,
            Edge::Draw => Action::Draw(game.draw()),
            Edge::Call => Action::Call(game.to_call()),
            Edge::Shove => Action::Shove(game.to_shove()),
            Edge::Raise(odds) => {
                let min = game.to_raise();
                let max = game.to_shove();
                let pot = game.pot() as Utility;
                let odd = Utility::from(*odds);
                let bet = (pot * odd) as Chips;
                match bet {
                    bet if bet >= max => Action::Shove(max),
                    bet if bet <= min => Action::Raise(min),
                    _ => Action::Raise(bet),
                }
            }
        }
    }
    /// generalization of mapping a concrete Action into a set of abstract Vec<Edge>
    /// this is mostly useful for enumerating a set of desired Raises
    /// which can be generated however.
    /// the contract is that the Actions returned by Game are legal,
    /// but the Raise amount can take any value >= the minimum provided by Game.
    fn edgeifies(&self, action: Action) -> Vec<Edge> {
        if let Action::Raise(_) = action {
            self.raises().into_iter().map(Edge::from).collect()
        } else {
            vec![Edge::from(action)]
        }
    }
    /// returns a set of possible raises given the current history
    /// we truncate in a few cases:
    /// - prevent N-betting explosion of raises
    /// - allow for finer-grained exploration in early streets
    /// - on the last street, restrict raise amounts so smaller grid
    fn raises(&self) -> Vec<Odds> {
        let n = self.subgame().iter().filter(|e| e.is_raise()).count();
        if n > crate::MAX_RAISE_REPEATS {
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
    /// returns the subgame history of the current node
    /// within the same Street of action.
    /// this should be made lazily in the future
    fn subgame(&self) -> Vec<Edge> {
        self.history()
            .into_iter()
            .take_while(|e| e.is_choice())
            .take(crate::MAX_DEPTH_SUBGAME)
            .copied()
            .collect()
    }
    /// we filter out the Draw actions from history
    /// and use it to lookup into our big policy
    /// lookup table, indexed by Bucket := (Path, Abs, Path)
    fn recall(&self) -> Vec<Edge> {
        self.history()
            .into_iter()
            .take(crate::MAX_DEPTH_SUBGAME)
            .copied()
            .collect()
    }

    /// if we were to play this edge, what would the
    /// history: Vec<Edge> of the resulting Node be?
    ///
    /// this used to be useful when the approach was
    /// to generate a Bucket for a Node>Data at
    /// *creation-time*, but now we generate the Bucket at
    /// *insertion-time* in Tree::attach()/Tree::insert().
    /// the current contract is that Data will be bucket-less
    /// until it gets inserted into a Tree. we could use
    /// different types for pre-post insertion, but this works
    /// well-enough to have Data own an Option<Bucket>.
    #[allow(unused)]
    fn extend(&self, edge: &Edge) -> Vec<Edge> {
        self.history()
            .into_iter()
            .rev()
            .chain(std::iter::once(edge))
            .rev()
            .take_while(|e| e.is_choice())
            .take(crate::MAX_DEPTH_SUBGAME)
            .copied()
            .collect()
    }
}

impl std::fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "N{}", self.index().index())
    }
}
