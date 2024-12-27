use super::bucket::Bucket;
use super::odds::Odds;
use super::path::Path;
use super::player::Player;
use crate::cards::street::Street;
use crate::gameplay::action::Action;
use crate::gameplay::game::Game;
use crate::gameplay::ply::Ply;
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

    // Navigational methods

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
        let choices = Path::from(self.fresh_continuations()); // could be from &'tree [Edge]
        Bucket::from((subgame, present, choices))
    }

    /// determine the set of branches that could be taken from this node
    /// this determines what Bucket we end up in since Tree::attach()
    /// uses this to assign Buckets to Data upon insertion
    pub fn branches(&self) -> Vec<(Edge, Game)> {
        self.stale_continuations()
            .into_iter()
            .map(|e| (e, self.actionization(&e)))
            .map(|(e, a)| (e.clone(), self.data().game().apply(a)))
            .collect()
    }
    /// what if we got the node continuations FROM the node data path bucket ?
    ///
    fn stale_continuations(&self) -> Vec<Edge> {
        Vec::<Edge>::from(self.data().bucket().2.clone())
    }
    /// returns the set of all possible actions from the current node
    /// this is useful for generating a set of children for a given node
    /// broadly goes from Node -> Game -> Action -> Edge
    fn fresh_continuations(&self) -> Vec<Edge> {
        self.data()
            .game()
            .legal()
            .into_iter()
            .flat_map(|a| self.edgifications(a))
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
    fn actionization(&self, edge: &Edge) -> Action {
        let game = self.data().game();
        match &edge {
            Edge::Check => Action::Check,
            Edge::Fold => Action::Fold,
            Edge::Draw => Action::Draw(game.draw()),
            Edge::Call => Action::Call(game.to_call()),
            Edge::Shove => Action::Shove(game.to_shove()),
            Edge::Raise(o) => {
                let min = game.to_raise();
                let max = game.to_shove();
                let bet = (game.pot() as Utility * Utility::from(*o)) as Chips;
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
    fn edgifications(&self, action: Action) -> Vec<Edge> {
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
        if n > crate::N_RAISE {
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
    fn chained(&self, edge: &Edge) -> Vec<Edge> {
        self.history()
            .into_iter()
            .rev()
            .chain(std::iter::once(edge))
            .rev()
            .take_while(|e| e.is_choice())
            .copied()
            .collect()
    }
}

impl std::fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "N{}", self.index().index())
    }
}
