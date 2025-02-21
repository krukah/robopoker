use super::bucket::Bucket;
use super::path::Path;
use super::player::Player;
use crate::gameplay::action::Action;
use crate::gameplay::game::Game;
use crate::gameplay::ply::Turn;
use crate::mccfr::data::Data;
use crate::mccfr::edge::Edge;
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
            .map(|e| (e, self.data().game().actionize(&e)))
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
            .map(|a| self.expand(a))
            .flatten()
            .collect()
    }

    /// generalization of mapping a concrete Action into a set of abstract Vec<Edge>
    /// this is mostly useful for enumerating a set of desired Raises
    /// which can be generated however.
    /// the contract is that the Actions returned by Game are legal,
    /// but the Raise amount can take any value >= the minimum provided by Game.
    fn expand(&self, action: Action) -> Vec<Edge> {
        match action {
            Action::Raise(_) => self
                .data()
                .game()
                .raises(self.subgame().iter().filter(|e| e.is_aggro()).count())
                .into_iter()
                .map(Edge::from)
                .collect(),
            _ => vec![Edge::from(action)],
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
