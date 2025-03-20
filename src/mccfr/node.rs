use super::bucket::Bucket;
use super::data::Data;
use super::path::Path;
use super::player::Player;
use crate::gameplay::edge::Edge;
use crate::gameplay::game::Game; // referenced in ::branches()
use crate::gameplay::ply::Turn; // referenced in ::payoff()
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
    pub fn previous(&self) -> Option<(Node<'tree>, &'tree Edge)> {
        match (self.parent(), self.incoming()) {
            (Some(parent), Some(incoming)) => Some((parent, incoming)),
            (Some(_), _) => unreachable!("live by the ship die by the ship"),
            (_, Some(_)) => unreachable!("live by the ship die by the ship"),
            (None, None) => None,
        }
    }
    pub fn outgoing(&self) -> Vec<&'tree Edge> {
        self.graph()
            .edges_directed(self.index(), Outgoing)
            .map(|edge| edge.weight())
            .collect()
    }
    fn incoming(&self) -> Option<&'tree Edge> {
        self.graph()
            .edges_directed(self.index(), Incoming)
            .next()
            .map(|edge| edge.weight())
    }
    pub fn parent(&self) -> Option<Node<'tree>> {
        self.graph()
            .neighbors_directed(self.index(), Incoming)
            .next()
            .map(|index| self.at(index))
    }
    pub fn children(&self) -> Vec<Node<'tree>> {
        self.graph()
            .neighbors_directed(self.index(), Outgoing)
            .map(|index| self.at(index))
            .collect()
    }
    pub fn follow(&self, edge: &Edge) -> Option<Node<'tree>> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.at(child.index()))
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

    fn at(&self, index: NodeIndex) -> Node<'tree> {
        Self::from((index, self.graph()))
    }

    /// this is the bucket that we use to lookup the Policy
    /// it is the product of the present abstraction, the
    /// history of choices leading up to the node, and the
    /// set of available continuations from the node.
    pub fn realize(&self) -> Bucket {
        let history = Path::from(self.recall());
        let present = self.data().abstraction().clone();
        let choices = self.choices();
        Bucket::from((history, present, choices))
    }

    /// determine the set of branches that could be taken from this node
    /// this determines what Bucket we end up in since Tree::attach()
    /// uses this to assign Buckets to Data upon insertion
    pub fn branches(&self) -> Vec<(Edge, Game)> {
        Vec::<Edge>::from(self.data().bucket().2.clone())
            .into_iter()
            .map(|e| (e, self.data().game().actionize(&e)))
            .map(|(e, a)| (e.clone(), self.data().game().apply(a)))
            .collect()
    }
    /// returns the set of all possible actions from the current node
    /// this is useful for generating a set of children for a given node
    /// broadly goes from Node -> Game -> Action -> Edge
    fn choices(&self) -> Path {
        self.data()
            .game()
            .choices(self.subgame().iter().filter(|e| e.is_aggro()).count())
            .into()
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
