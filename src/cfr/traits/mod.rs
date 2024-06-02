#![allow(dead_code)]
pub(crate) mod marker;
pub(crate) mod training;
pub(crate) mod tree;

pub(crate) type Utility = f32;
pub(crate) type Probability = f32;

use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::Direction::Incoming;
use petgraph::Direction::Outgoing;
use std::hash::Hash;

trait Player<'t>: 't + Sized + Eq {}
trait Bucket<'t>: 't + Sized + Eq + Hash {}
trait Action<'t>: 't + Sized + Eq + Hash + Copy {}

/// the inner state of a node, abstracted over the type of action and bucket
trait Vertex<'t, A, B, C>
where
    Self: 't + Sized,
    A: Action<'t>,
    B: Bucket<'t>,
    C: Player<'t>,
{
    // required
    fn root() -> Self;
    fn expand(&'t self) -> Vec<(Self, A)>;
    fn bucket(&'t self) -> &'t B;
    fn player(&'t self) -> &'t C;
    fn payoff(&'t self, player: &'t C) -> Utility;
}

/// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
trait Node<'t, V, A, B, C>
where
    Self: 't + Sized,
    V: Vertex<'t, A, B, C>,
    A: Action<'t>,
    B: Bucket<'t>,
    C: Player<'t>,
{
    // required
    fn local(&'t self) -> &'t V;
    fn index(&'t self) -> &'t NodeIndex;
    fn graph(&'t self) -> &'t DiGraph<Self, A>;

    // obserability (implemented from Vertex)
    fn payoff(&'t self, player: &'t C) -> Utility {
        self.local().payoff(player)
    }
    fn bucket(&'t self) -> &'t B {
        self.local().bucket()
    }
    fn player(&'t self) -> &'t C {
        self.local().player()
    }

    // walkability
    fn parent(&'t self) -> Option<&'t Self> {
        self.graph()
            .neighbors_directed(*self.index(), Incoming)
            .next()
            .map(|index| {
                self.graph()
                    .node_weight(index)
                    .expect("tree property: if incoming edge, then parent")
            })
    }
    fn children(&'t self) -> Vec<&'t Self> {
        self.graph()
            .neighbors_directed(*self.index(), Outgoing)
            .map(|c| {
                self.graph()
                    .node_weight(c)
                    .expect("tree property: if outgoing edge, then child")
            })
            .collect()
    }
    fn incoming(&'t self) -> Option<&'t A> {
        self.graph()
            .edges_directed(*self.index(), Incoming)
            .next()
            .map(|e| e.weight())
    }
    fn outgoing(&'t self) -> Vec<&'t A> {
        self.graph()
            .edges_directed(*self.index(), Outgoing)
            .map(|e| e.weight())
            .collect()
    }
    fn descendants(&'t self) -> Vec<&'t Self> {
        match self.children().len() {
            0 => vec![&self],
            _ => self
                .children()
                .iter()
                .map(|child| child.descendants())
                .flatten()
                .collect(),
        }
    }
    fn follow(&'t self, edge: &'t A) -> &'t Self {
        self.children()
            .iter()
            .find(|child| edge == child.incoming().unwrap())
            .unwrap()
    }
}

/// distribution over indistinguishable nodes, abstracted over the type of node
trait Info<'t, N, V, A, B, C>
where
    N: Node<'t, V, A, B, C>,
    V: Vertex<'t, A, B, C>,
    A: Action<'t>,
    B: Bucket<'t>,
    C: Player<'t>,
{
    // required
    fn roots(&'t self) -> &'t Vec<&'t N>;

    fn bucket(&'t self) -> &'t B {
        self.roots().iter().next().unwrap().bucket()
    }
    fn player(&'t self) -> &'t C {
        self.roots().iter().next().unwrap().player()
    }
    fn outgoing(&'t self) -> Vec<&'t A> {
        self.roots().iter().next().unwrap().outgoing()
    }
}

/// a tree will own the graph and infosets
trait Tree<'t, I, N, V, A, B, C>
where
    I: Info<'t, N, V, A, B, C>,
    N: Node<'t, V, A, B, C>,
    V: Vertex<'t, A, B, C>,
    A: Action<'t>,
    B: Bucket<'t>,
    C: Player<'t>,
{
    // required
    fn infosets(&'t self) -> &'t Vec<&'t I>;
}

/// a policy π is a distribution over actions given a bucket. Equivalently a vector indexed by action ∈ A
trait Distribution<'t, A>
where
    A: Action<'t>,
{
    // required
    fn weight(&self, action: &A) -> Probability;
    fn sample(&self) -> &A;
}

/// a strategy σ is a policy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
trait Strategy<'t, D, A, B>
where
    D: Distribution<'t, A>,
    A: Action<'t>,
    B: Bucket<'t>,
{
    // required
    fn policy(&self, bucket: &B) -> &D;
}

/// a profile σ consists of a strategy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
trait Profile<'t, S, D, N, V, A, B, C>
where
    S: Strategy<'t, D, A, B>,
    D: Distribution<'t, A>,
    N: Node<'t, V, A, B, C>,
    V: Vertex<'t, A, B, C>,
    A: Action<'t>,
    B: Bucket<'t>,
    C: Player<'t>,
{
    // required
    fn strategy(&self, player: &C) -> &S;

    // provided
    fn gain(&self, root: &'t N, action: &'t A) -> Utility {
        self.cfactual_value(root, action) - self.expected_value(root)
    }
    fn cfactual_value(&self, root: &'t N, action: &'t A) -> Utility {
        self.cfactual_reach(root)
            * root //                                       suppose you're here on purpose, counterfactually
                .follow(action) //                          suppose you're here on purpose, counterfactually
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&self, root: &'t N) -> Utility {
        self.expected_reach(root)
            * root
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&self, root: &'t N, leaf: &'t N) -> Utility {
        leaf.payoff(root.player())
            * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
    }
    // probability calculations
    fn weight(&self, node: &'t N, action: &A) -> Probability {
        self.strategy(node.player())
            .policy(node.bucket())
            .weight(action)
    }
    fn cfactual_reach(&self, node: &'t N) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.cfactual_reach(parent)
                    * if node.player() == parent.player() {
                        1.0
                    } else {
                        self.weight(
                            parent,
                            node.incoming().expect("if has parent, then has incoming"),
                        )
                    }
            }
        }
    }
    fn expected_reach(&self, node: &'t N) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.expected_reach(parent)
                    * self.weight(
                        parent,
                        node.incoming().expect("if has parent, then has incoming"),
                    )
            }
        }
    }
    fn relative_reach(&self, root: &'t N, leaf: &'t N) -> Probability {
        //? gotta optimize out integration over shared ancestors that cancels out in this division. Node: Eq? Hash?
        self.expected_reach(leaf) / self.expected_reach(root)
    }
    fn sampling_reach(&self, _: &'t N, _: &'t N) -> Probability {
        1.0
    }
}

/// an optimizer updates profile to minimize regret, and updates regrets from existing profiles.
trait Optimizer<'t, P, S, D, I, N, V, A, B, C>
where
    P: Profile<'t, S, D, N, V, A, B, C>,
    S: Strategy<'t, D, A, B>,
    D: Distribution<'t, A>,
    I: Info<'t, N, V, A, B, C>,
    N: Node<'t, V, A, B, C>,
    V: Vertex<'t, A, B, C>,
    A: Action<'t>,
    B: Bucket<'t>,
    C: Player<'t>,
{
    // required
    fn profile(&self) -> &P;
    fn update_regret(&mut self, info: &I);
    fn update_policy(&mut self, info: &I);
    fn current_regret(&self, info: &'t I, action: &'t A) -> Utility;
    fn instant_regret(&self, info: &'t I, action: &'t A) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.profile().gain(root, action))
            .sum::<Utility>()
    }
    fn pending_regret(&self, info: &'t I, action: &'t A) -> Utility {
        self.instant_regret(info, action) + self.current_regret(info, action)
    }
    fn policy_vector(&self, info: &'t I) -> Vec<(A, Probability)> {
        let regrets = info
            .roots()
            .iter()
            .map(|root| root.outgoing())
            .flatten()
            .map(|action| (*action, self.current_regret(info, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(A, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
    }
    fn regret_vector(&self, info: &'t I) -> Vec<(A, Utility)> {
        info.outgoing()
            .iter()
            .map(|action| (**action, self.pending_regret(info, action)))
            .collect()
    }
}

/// trainer will update regrets and profile in a mutable loop
trait Trainer<'t, O, P, S, D, T, I, N, V, A, B, C>
where
    O: Optimizer<'t, P, S, D, I, N, V, A, B, C>,
    P: Profile<'t, S, D, N, V, A, B, C>,
    S: Strategy<'t, D, A, B>,
    D: Distribution<'t, A>,
    T: Tree<'t, I, N, V, A, B, C>,
    I: Info<'t, N, V, A, B, C>,
    N: Node<'t, V, A, B, C>,
    V: Vertex<'t, A, B, C>,
    A: Action<'t>,
    B: Bucket<'t>,
    C: Player<'t>,
{
    // required
    fn train(&mut self, n: usize);
}
