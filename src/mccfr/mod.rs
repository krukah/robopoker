#![allow(dead_code)]

use crate::transport::density::Density;
use crate::Probability;
use crate::Utility;

pub mod blueprint;
pub mod bucket;
pub mod counterfactual;
pub mod data;
pub mod discount;
pub mod edge;
pub mod encoder;
pub mod info;
pub mod memory;
pub mod node;
pub mod odds;
pub mod partition;
pub mod path;
pub mod phase;
pub mod player;
pub mod policy;
pub mod profile;
pub mod recall;
pub mod regret;
pub mod strategy;
pub mod tree;

trait Spot: Clone + Copy {}
trait Turn: Clone + Copy {}
trait Head: Clone + Copy {}
trait Game: Clone + Copy {}
trait Edge: Clone + Copy {}

trait Jump: Clone + Copy + Node + Edge {}
trait Leaf: Clone + Copy + Head + Edge + Game {}

/// trees handle the recursive growth of their
/// underlying graph data structure. all we need
/// is a root (Node) and some forks (Edge, Node)
/// to recursively insert new Games.
trait Tree: Clone + Copy + Default {
    /// initial insertion of root node
    /// must have nothing to do with
    /// topology.
    fn seed<N, I, G>(&mut self, info: I, seed: G) -> N
    where
        I: Info,
        G: Game,
        N: Node;

    /// this is the only method by which
    /// the tree can grow, is if we know:
    /// what Game we're attaching,
    /// what Node to attach a new Game,
    /// what Edge to connect it with
    fn grow<N, I, L>(&mut self, info: I, leaf: L) -> N
    where
        L: Leaf,
        N: Node;
}

/// this is more like Bucket than it is InfoSet.
/// it's just the copyable struct. it is NOT the
/// collection of Nodes that share the same Bucket.
/// that is gonna be something else, part of the Partition.
trait Info: Clone + Copy {
    /// constructor
    fn from<C, E, H>(choices: C, chances: H) -> Self
    where
        H: Spot,
        E: Edge,
        C: Iterator<Item = E>;

    /// what is the abstraction at this Infoset?
    fn chances<H>(&self) -> &H
    where
        H: Spot;

    /// what are the choices available at this Infoset?
    fn choices<C, E>(&self) -> &C
    where
        E: Edge,
        C: Iterator<Item = E>;
}

/// local topology accessible via walkable tree functions.
/// interior data is exposed via game reference. topology is
/// exposed via tree reference. this trait only handles
/// navigation methods.
trait Node: Clone + Copy {
    /// this will go one step up in the tree
    /// w.r.t. another Edge AND another Node
    fn parent<J>(&self) -> Option<J>
    where
        J: Jump;

    /// this will go one step down in the tree
    /// w.r.t. all other Nodes
    fn children<W>(&self) -> W
    where
        W: Iterator<Item = Self>;

    /// this will go down as many steps as possible,
    /// recursing through children until we reach terminal nodes
    fn descendants<W>(&self) -> W
    where
        W: Iterator<Item = Self>;

    /// this will go up as many steps as possible,
    /// recursing through parents until we reach the root node
    fn ancestors<B, J>(&self) -> B
    where
        J: Jump,
        B: Iterator<Item = J>;

    /// reveal enclosing tree by reference. we can probably assume
    /// that the implementation will yield a Node that is
    /// owned by this Tree Graph, and thus we would
    /// also have some lifetime parameters.
    fn tree<T>(&self) -> &T
    where
        T: Tree;

    /// reveal interior data storage
    /// by reference, assuming it cannot
    /// change after tree insertion.
    fn game<G>(&self) -> &G
    where
        G: Game;

    /// lookup the pre-computed information
    fn info<I>(&self) -> &I
    where
        I: Info;
}

/// all of our fat lookup tables are encapsulated
/// by the Encoder. Isomorphism -> Abstraction lookups
/// and Info generatoin all happen here.
trait Encoder {
    /// lookup the infoset for a game state that
    /// will become the root of the Tree. you don't
    /// need a Tree because you know the topology of the
    /// empty Tree you're gonna grow later.
    fn root<I, G>(&self, game: &G) -> I
    where
        I: Info,
        G: Game;

    /// for non-root Nodes, we should look at *where* we are
    /// attaching to the Tree to decide *how* we should attach it.
    /// (if depth-limited, or action-constrained, etc.)
    fn info<I, T, L>(&self, tree: &T, leaf: &L) -> I
    where
        I: Info,
        T: Tree,
        L: Leaf;
}

/// to get weights for edges under certain
/// information sets, we need to be able to map
/// buckets/infosets to policies, i.e.
/// Bucket -> Policy -> Edge -> Probability
trait Profile {
    /// tell me: what is the Probability of visiting this
    /// particular Edge, conditional on being in this Infoset?
    fn policy<P, I, E>(&self, info: &I) -> P
    where
        I: Info,
        E: Edge,
        P: Density<Support = E>;

    /// add a new Infoset/Bucket to the Profile
    /// when we encounter it for the first time
    fn witness<I>(&mut self, info: &I)
    where
        I: Info;

    /// update myself
    fn update_regret<P, I, E>(&mut self, info: &I, update: &P)
    where
        I: Info,
        E: Edge,
        P: Density<Support = E>;

    /// update myself
    fn update_policy<P, I, E>(&mut self, info: &I, update: &P)
    where
        I: Info,
        E: Edge,
        P: Density<Support = E>;

    /// from this Node, what is the Probability of
    /// traversing this particular Edge? because we do traversal
    /// we need Node to access global from local topology.
    fn reach_children<N, E>(&self, node: N, edge: E) -> Probability
    where
        N: Node,
        E: Edge;

    /// conditional on being in a given Infoset,
    /// what is the Probability of
    /// visiting this particular leaf Node,
    /// given the distribution offered by Profile?
    fn reach_relative<N>(&self, head: N, tail: N) -> Probability
    where
        N: Node;

    /// if we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn reach_internal<N>(&self, node: N) -> Probability
    where
        N: Node;

    /// if, counterfactually, we had played toward this infoset,
    /// then what would be the Probability of us being n this infoset?
    /// i.e. assuming our opponents played according to distributions from Profile, but we did not.
    ///
    /// this function also serves as a form of importance sampling.
    /// MCCFR requires we adjust our reach in counterfactual
    /// regret calculation to account for the under- and over-sampling
    /// of regret across different Infosets.
    fn reach_external<N>(&self, node: N) -> Probability
    where
        N: Node;
}

/// the Sampler will encapsulate all the massive objects
/// that we need to generate Trees in memory
/// (namely, Encoder + Profile). it will also handle
/// the mutability constraints of profile witnessing.
trait Sampler {
    /// to support different sampling schemes, we need
    /// to assign a Player to be the "traverser" of the
    /// Tree that we are harvesting.
    fn traverser<T>(&self) -> &T
    where
        T: Turn;

    /// staticish reference to the massive lookup table
    /// of Observation -> Abstraction
    fn encoder<E>(&self) -> &E
    where
        E: Encoder;

    /// we'll probably need to have a Profile
    /// to correctly sample Node, Edge pairs
    /// based on different sampling schemes
    fn profile<P>(&self) -> &P
    where
        P: Profile;

    /// encapsulation of [external, internal, probing]
    /// sampling strategies. in practice, we will use
    /// a Profile to sample different forks from this
    /// Node according to existing policy.
    ///
    /// this might need to be mutable if we
    /// include Profile::witness() within here
    fn expand<N, I, L>(&self, node: N) -> I
    where
        N: Node,
        L: Leaf,
        I: Iterator<Item = L>;

    /// just grow a tree from the ground up
    fn sample<T>(&self) -> T
    where
        T: Tree;
}

/// this trait is used to evaluate the Utility of
/// a given < Strategy | Tree > "inner product", which
/// i'm calling marginal_value here for whatever reason.
/// the &Tree is embedded in the &Node, and the &Policy
/// are embedded in the &Profile. we only walk the Tree
/// and so don't need ownership of anything other than our profile reference..
/// so consider this a wrapper around Profile.
trait Trainer {
    /// we must reference a pre-computed Strategy
    /// from which we calculate transition probabilities.
    /// reach flows -> down the tree
    /// value flows <-   up the tree
    fn profile<P>(&self) -> &P
    where
        P: Profile;

    /// using our current strategy Profile,
    /// compute the regret vector
    /// by calculating the marginal Utitlity
    /// missed out on for not having followed
    /// every walkable Edge at this Infoset/Node/Bucket
    fn regret_vector<N, R, I, E>(&self, infoset: &I) -> R
    where
        N: Node,
        E: Edge,
        I: Iterator<Item = N>,
        R: Density<Support = E>;

    /// using our current regret Profile,
    /// compute a new strategy vector
    /// by following a given Edge
    /// proportionally to how much regret we felt
    /// for not having followed that Edge in the past.
    fn policy_vector<N, P, I, E>(&self, infoset: &I) -> P
    where
        N: Node,
        E: Edge,
        I: Iterator<Item = N>,
        P: Density<Support = E>;

    /// assuming we start at a given head Node,
    /// and that we sample the tree according to Profile,
    /// how much Utility does
    /// this leaf Node backpropagate up to us?
    fn relative_value<N>(&self, head: N, tail: N) -> Utility
    where
        N: Node;

    /// assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon
    /// visiting this Node?
    fn expected_value<N>(&self, head: N) -> Utility
    where
        N: Node;

    /// if, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value<N>(&self, tail: N) -> Utility
    where
        N: Node;

    /// if at this given head Node,
    /// we diverged from our Profile strategy
    /// by "playing toward" this Infoset
    /// and following this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn marginal_value<N>(&self, tail: N) -> Utility
    where
        N: Node;

    /// historically,
    /// upon visiting any Node inthis Infoset,
    /// how much cumulative Utility have we missed out on
    /// for not having followed this Edge?
    fn running_regret<I, E, N>(&self, edge: &E, infoset: &I) -> Utility
    where
        N: Node,
        E: Edge,
        I: Iterator<Item = N>;

    /// conditional on being in this Infoset,
    /// distributed across all its head Nodes,
    /// with paths weighted according to our Profile:
    /// if we follow this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn instant_regret<I, E, N>(&self, edge: &E, infoset: &I) -> Utility
    where
        N: Node,
        E: Edge,
        I: Iterator<Item = N>;
}
