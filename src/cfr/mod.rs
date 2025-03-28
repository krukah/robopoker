pub mod traits;

/// the turn is fully abstracted. it is almost a marker trait,
/// but requires two fundamnetal properties:
/// 1. it has an element of chance
/// 2. its elements can be mapped to by unsigned integers
trait Turn: Clone + Copy + PartialEq + Eq + std::hash::Hash {}

/// the edge is fully abstracted. it is basically a marker trait
trait Edge: Copy + Clone + PartialEq + Eq + crate::transport::support::Support {}

/// the information bucket is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to determine:
///  what possible Edges may emerge from this Node (Decision)
///
/// the generation of this information is the responsibility of the Encoder,
/// which has global tree context and may make probabilistic or path-dependent decisions
trait Info: Clone + Copy + PartialEq + Eq + std::hash::Hash {
    type E: Edge;
    type T: Turn;
    fn choices(&self) -> Vec<Self::E>;
}

/// the tree-local game state is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to create a Game from:
///  scratch (i.e. root node without context)
///  
/// the implementation must be able to determine:
///  whose turn is it (have a Player function)
///  how much payoff for each player (only must be defined for leaf nodes)
///
/// it is up to the implementation of Encoder to decide how the
/// game tree is navigated, in a tree-non-local context. this Game
/// structure should only concern itself of local properties.
trait Game: Clone + Copy {
    type E: Edge;
    type T: Turn;
    fn root() -> Self;
    fn turn(&self) -> Self::T;
    fn payoff(&self, player: Self::T) -> crate::Utility;
}

/// this is pre-implemented. it is a distrubtion
/// over policy space. i.e., it is a Density over Edges,
/// presumably at a given Info point.
type Policy<E> = Vec<(E, crate::Probability)>;

/// just a wrapper for child nodes that haven't yet
/// been sampled from, so it's half Node (parent)
/// and half Game (child) with the "birthing" Edge
/// thrown in there too. everything an Encoder needs to
/// make some children.
type Branch<E, G> = (E, G, petgraph::graph::NodeIndex);

/// this is pre-implemented. it is a wrapper around
/// different edge-indexed distributions of regret and policy
/// at a given Info point.
///
/// this is the smallest unit of information that can be used
/// to update a Profile. two densities over decision space.
type Counterfactual<E, I> = (I, Policy<E>, Policy<E>);

/// the infoset is pre-implemented. it is an [un]ordered collection of
/// Node indices, and a thread-safe readonly reference to the Tree
/// in which they reside.
///
/// because Node's preserve lifetime and full-tree reference,
/// we are able to walk in and out of this InfoSet without
/// having to allocate anything real. Node's are cheap.
#[derive(Debug, Default)]
struct InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    root: Vec<petgraph::graph::NodeIndex>,
    tree: std::sync::Arc<Tree<T, E, G, I>>,
}
impl<T, E, G, I> InfoSet<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<T = T, E = E>,
    I: Info<E = E, T = T>,
{
    fn from(tree: std::sync::Arc<Tree<T, E, G, I>>) -> Self {
        Self {
            root: Vec::new(),
            tree,
        }
    }
    fn next(&mut self) -> Option<Node<T, E, G, I>> {
        self.root.pop().map(|i| self.tree.at(i))
    }
    fn push(&mut self, index: petgraph::graph::NodeIndex) {
        self.root.push(index);
    }
    fn span(&self) -> Vec<Node<T, E, G, I>> {
        self.root.iter().copied().map(|i| self.tree.at(i)).collect()
    }
    fn head(&self) -> Node<T, E, G, I> {
        self.tree
            .at(self.root.first().copied().expect("nodes in info"))
    }
    fn info(&self) -> I {
        self.head().info().clone()
    }
}

/// the node is pre-implemented. it is a wrapper around
/// a petgraph::graph::NodeIndex, and a thread-safe readonly reference
/// to the Tree in which it resides.
///
/// by only assuming the tree property of the underlying graph,
/// we can implement navigation methods recursively. all while being
/// fully generic over Turn Edge Game Info. just that they need to be
#[derive(Copy, Clone)]
struct Node<'tree, T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    index: petgraph::graph::NodeIndex,
    graph: &'tree petgraph::graph::DiGraph<(G, I), E>,
    danny: std::marker::PhantomData<(T, I)>,
}
impl<'tree, T, E, G, I> Node<'tree, T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn index(&self) -> petgraph::graph::NodeIndex {
        self.index
    }
    fn graph(&self) -> &'tree petgraph::graph::DiGraph<(G, I), E> {
        self.graph
    }
    fn game(&self) -> &G {
        &self
            .graph
            .node_weight(self.index())
            .expect("valid game index")
            .0
    }
    fn info(&self) -> &I {
        &self
            .graph
            .node_weight(self.index())
            .expect("valid info index")
            .1
    }
    fn at(&self, index: petgraph::graph::NodeIndex) -> Node<'tree, T, E, G, I> {
        Self {
            index,
            graph: self.graph(),
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }

    fn next(&self) -> Option<(Node<'tree, T, E, G, I>, &'tree E)> {
        match (self.parent(), self.incoming()) {
            (Some(parent), Some(incoming)) => Some((parent, incoming)),
            (Some(_), _) => unreachable!("live by the ship die by the ship"),
            (_, Some(_)) => unreachable!("live by the ship die by the ship"),
            (None, None) => None,
        }
    }
    fn parent(&self) -> Option<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|index| self.at(index))
    }
    fn incoming(&self) -> Option<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Incoming)
            .next()
            .map(|edge| edge.weight())
    }
    fn follow(&self, edge: &E) -> Option<Node<'tree, T, E, G, I>> {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .map(|child| self.at(child.index()))
    }
    fn outgoing(&self) -> Vec<&'tree E> {
        self.graph()
            .edges_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|edge| edge.weight())
            .collect()
    }
    fn children(&self) -> Vec<Node<'tree, T, E, G, I>> {
        self.graph()
            .neighbors_directed(self.index(), petgraph::Direction::Outgoing)
            .map(|index| self.at(index))
            .collect()
    }
    fn descendants(&self) -> Vec<Node<'tree, T, E, G, I>> {
        if self.children().is_empty() {
            vec![self.clone()]
        } else {
            self.children()
                .into_iter()
                .map(|child| child.descendants())
                .flatten()
                .collect()
        }
    }
}

/// the tree is pre-implemented. it is a wrapper around
/// a petgraph::graph::DiGraph. at each vertex, we store a
/// tuple of the fully abstracted Game and Info.
///
/// we assume that we are generated recursively from Encoder and Profile.
/// together, these traits enable "exploring the game space" up to the
/// rules of the game, i.e. implementation of T, E, G, I, Encoder, Profile.
#[derive(Debug, Default)]
struct Tree<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    graph: petgraph::graph::DiGraph<(G, I), E>,
    danny: std::marker::PhantomData<(T, I)>,
}
impl<T, E, G, I> Tree<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn empty() -> Self {
        Self {
            graph: petgraph::graph::DiGraph::new(),
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }
    /// get all Nodes in the Tree
    fn all(&self) -> impl Iterator<Item = Node<T, E, G, I>> {
        self.graph.node_indices().map(|n| self.at(n))
    }
    /// get a Node by index
    fn at(&self, index: petgraph::graph::NodeIndex) -> Node<T, E, G, I> {
        Node {
            index,
            graph: &self.graph,
            danny: std::marker::PhantomData::<(T, I)>,
        }
    }
    /// seed a Tree by giving an (Info, Game) and getting a Node
    fn seed(&mut self, info: I, seed: G) -> Node<T, E, G, I> {
        let seed = self.graph.add_node((seed, info));
        self.at(seed)
    }
    /// extend a Tree by giving a Leaf and getting a Node
    fn grow(&mut self, info: I, leaf: Branch<E, G>) -> Node<T, E, G, I> {
        let tail = self.graph.add_node((leaf.1, info));
        let edge = self.graph.add_edge(leaf.2, tail, leaf.0);
        assert!(edge.index() == tail.index() - 1);
        self.at(tail)
    }
    /// group non-leaf Nodes by Info into InfoSets
    fn partition(self) -> std::collections::HashMap<I, InfoSet<T, E, G, I>> {
        let tree = std::sync::Arc::new(self);
        let mut info = std::collections::HashMap::new();
        for node in tree.all().filter(|n| n.children().len() > 0) {
            info.entry(node.info().clone())
                .or_insert_with(|| InfoSet::from(tree.clone()))
                .push(node.index());
        }
        info
    }
}

/// infoset encoding is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the encoder must be able to create an Info from:
///  a Game
///  a Game, an Edge, and the parent Node for topological context
///
/// some implemenstaions may not need to reference the parent Node,
/// RPS for example has trivial infoset encoding,
/// whereas NLHE must learn the massive abstraction from kmeans clustering
/// over the set of all hands up to strategic isomorphism.
trait Encoder<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    fn seed(&self, game: &G) -> I;
    fn info(&self, tree: &Tree<T, E, G, I>, leaf: Branch<E, G>) -> I;
    fn grow(&self, node: &Node<T, E, G, I>) -> Vec<Branch<E, G>>;
}

/// the strategy is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to determine:
///  what is the Density over the Edges
trait Profile<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    /// who's turn is it?
    fn walker(&self) -> T;
    /// lookup historical policy distribution, given this information
    fn policy(&self, info: &I) -> &Policy<E>;
    /// lookup historical regret value, given this information
    fn regret(&self, info: &I) -> &Policy<E>;

    /// topology-based sampling. i.e. external, probing, targeted, uniform, etc.
    fn sample(&self, node: &Node<T, E, G, I>, branches: Vec<Branch<E, G>>) -> Vec<Branch<E, G>>;

    /// automatic

    /// Using our current strategy Profile,
    /// compute the regret vector
    /// by calculating the marginal Utitlity
    /// missed out on for not having followed
    /// every walkable Edge at this Infoset/Node/Bucket
    fn regret_vector(&self, infoset: &InfoSet<T, E, G, I>) -> Policy<E> {
        infoset
            .info()
            .choices()
            .into_iter()
            .map(|edge| (edge, self.info_gain(infoset, &edge)))
            .map(|(e, r)| (e, r.max(crate::REGRET_MIN)))
            .map(|(e, r)| (e, r.min(crate::REGRET_MAX)))
            .inspect(|(_, r)| assert!(!r.is_nan()))
            .inspect(|(_, r)| assert!(!r.is_infinite()))
            .collect::<Policy<E>>()
    }
    /// lookup historical policy distribution, given this information
    fn policy_vector(&self, infoset: &InfoSet<T, E, G, I>) -> Policy<E> {
        use crate::transport::density::Density;
        let regrets = infoset
            .info()
            .choices()
            .into_iter()
            .map(|e| (e, self.regret(&infoset.info()).density(&e)))
            .map(|(a, r)| (a, r.max(crate::POLICY_MIN)))
            .collect::<Policy<E>>();
        let denominator = regrets.iter().map(|(_, r)| r).sum::<crate::Utility>();
        let policy = regrets
            .into_iter()
            .map(|(a, r)| (a, r / denominator))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect::<Policy<E>>();
        policy
    }

    /// at the immediate location of this Node,
    /// what is the Probability of transitioning via this Edge?
    fn outgoing_reach(&self, node: Node<T, E, G, I>, edge: E) -> crate::Probability {
        use crate::transport::density::Density;
        self.policy(&node.info()).density(&edge)
    }
    /// Conditional on being in a given Infoset,
    /// what is the Probability of
    /// visiting this particular leaf Node,
    /// given the distribution offered by Profile?
    fn relative_reach(&self, root: Node<T, E, G, I>, leaf: Node<T, E, G, I>) -> crate::Probability {
        if root.index() == leaf.index() {
            1.0
        } else {
            match leaf.next() {
                None => unreachable!("leaf must be downstream of root"),
                Some((parent, incoming)) => {
                    self.relative_reach(root, parent) // .
                    * self.outgoing_reach(parent, *incoming)
                }
            }
        }
    }
    /// If we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn expected_reach(&self, root: Node<T, E, G, I>) -> crate::Probability {
        match root.next() {
            None => 1.0,
            Some((parent, incoming)) => {
                self.expected_reach(parent) // .
                * self.outgoing_reach(parent, *incoming)
            }
        }
    }
    /// If, counterfactually, we had played toward this infoset,
    /// then what would be the Probability of us being in this infoset?
    /// i.e. assuming our opponents played according to distributions from Profile, but we did not.
    ///
    /// This function also serves as a form of importance sampling.
    /// MCCFR requires we adjust our reach in counterfactual
    /// regret calculation to account for the under- and over-sampling
    /// of regret across different Infosets.
    fn cfactual_reach(&self, node: Node<T, E, G, I>) -> crate::Probability {
        match node.next() {
            None => 1.0,
            Some((parent, incoming)) => {
                if self.walker() != parent.game().turn() {
                    self.cfactual_reach(parent) * self.outgoing_reach(parent, *incoming)
                } else {
                    self.cfactual_reach(parent)
                }
            }
        }
    }

    /// Assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon
    /// visiting this Node?
    fn expected_value(&self, root: Node<T, E, G, I>) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        self.expected_reach(root)
            * root
                .descendants()
                .into_iter()
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<crate::Utility>()
    }
    /// relative to the player at the root Node of this Infoset,
    /// what is the Utility of this leaf Node?
    fn relative_value(&self, root: Node<T, E, G, I>, leaf: Node<T, E, G, I>) -> crate::Utility {
        leaf.game().payoff(root.game().turn()) // .
                                               // * self.relative_reach(root, leaf)
                                               // / self.cfactual_reach(leaf)
    }
    /// assuming we start at a given head Node,
    /// and that we sample the tree according to Profile,
    /// how much Utility does
    /// this leaf Node backpropagate up to us?
    fn intended_value(&self, root: Node<T, E, G, I>, leaf: Node<T, E, G, I>) -> crate::Utility {
        // should the relative reach calculation use head at all? may be double counted at self.cfactual.profile.cfactual_reach(head). maybe use expected_reach instead?
        assert!(self.walker() == root.game().turn());
        self.expected_value(root) // .
        * self.relative_reach(root, leaf) // .
        / self.cfactual_reach(leaf)
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(&self, root: Node<T, E, G, I>, edge: &E) -> crate::Utility {
        // maybe use expected_reach instead? cfactual_reach may double count at bayesian_value in numerator
        assert!(self.walker() == root.game().turn());
        self.cfactual_reach(root)
            * root
                .follow(edge)
                .expect("edge belongs to outgoing")
                .descendants()
                .into_iter()
                .map(|leaf| self.intended_value(root, leaf))
                .sum::<crate::Utility>()
    }

    /// Conditional on being in this Infoset,
    /// distributed across all its head Nodes,
    /// with paths weighted according to our Profile:
    /// if we follow this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn info_gain(&self, info: &InfoSet<T, E, G, I>, edge: &E) -> crate::Utility {
        info.span()
            .into_iter()
            .inspect(|root| assert!(self.walker() == root.game().turn()))
            .map(|root| self.node_gain(root, edge))
            .sum::<crate::Utility>()
    }
    /// Using our current strategy Profile, how much regret
    /// would we gain by following this Edge at this Node?
    fn node_gain(&self, root: Node<T, E, G, I>, edge: &E) -> crate::Utility {
        self.cfactual_value(root, edge) - self.expected_value(root)
    }
}

///
trait Sampler<T, E, G, I, X, Y>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
    X: Encoder<T, E, G, I>,
    Y: Profile<T, E, G, I>,
{
    fn encoder(&self) -> &X;
    fn profile(&self) -> &Y;

    /// LEVEL 3:  turn a bunch of infosets into a bunch of counterfactuals
    fn batch(&self) -> Vec<Counterfactual<E, I>> {
        self.infos()
            .into_iter()
            .map(|i| {
                (
                    i.info(),
                    self.profile().regret_vector(&i),
                    self.profile().policy_vector(&i),
                )
            })
            .collect()
    }
    /// LEVEL 2: turn a bunch of trees into a bunch of infosets
    fn infos(&self) -> Vec<InfoSet<T, E, G, I>> {
        self.trees()
            .into_iter()
            .map(|tree| tree.partition().into_values())
            .flatten()
            .filter(|infoset| infoset.head().game().turn() == self.profile().walker())
            .collect()
    }
    /// LEVEL 1: generate a bunch of trees to be partitioned into InfoSets downstream
    fn trees(&self) -> Vec<Tree<T, E, G, I>> {
        (0..crate::CFR_BATCH_SIZE)
            .map(|_| {
                let mut todo = Vec::new();
                let mut tree = Tree::<T, E, G, I>::empty();
                let root = G::root();
                let info = self.encoder().seed(&root);
                let node = tree.seed(info, root);
                let children = self.encoder().grow(&node);
                let children = self.profile().sample(&node, children);
                todo.extend(children);
                while let Some(leaf) = todo.pop() {
                    let info = self.encoder().info(&tree, leaf);
                    let node = tree.grow(info, leaf);
                    let children = self.encoder().grow(&node);
                    let children = self.profile().sample(&node, children);
                    todo.extend(children);
                }
                tree
            })
            .collect::<Vec<_>>()
    }
}

///
trait Solution<T, E, G, I, X, Y, Z>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
    X: Encoder<T, E, G, I>,
    Y: Profile<T, E, G, I>,
    Z: Sampler<T, E, G, I, X, Y>,
{
    fn encoder(&mut self) -> &mut X;
    fn profile(&mut self) -> &mut Y;
    fn sampler(&mut self) -> &mut Z;

    fn update_regret(&mut self, cfr: &Counterfactual<E, I>);
    fn update_policy(&mut self, cfr: &Counterfactual<E, I>);

    fn solve(&mut self) {
        for _ in 0..crate::CFR_ITERATIONS {
            for ref update in self.sampler().batch() {
                self.profile().update_regret(update);
                self.profile().update_policy(update);
            }
        }
    }
}
