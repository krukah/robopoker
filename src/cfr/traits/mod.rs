#![allow(dead_code)]

use std::hash::Hash;

pub(crate) mod marker;
pub(crate) mod training;
pub(crate) mod tree;

pub(crate) type Utility = f32;
pub(crate) type Probability = f32;

trait Action: Eq + Sized + Hash {}
trait Bucket: Eq + Sized + Hash {}
trait Player: Eq + Sized {}

// /// observable features of a unique history + game state, abstracted over the type of player and bucket
// trait Visible<E, B, Y>
// where
//     Self: Sized,
//     E: Action,
//     B: Bucket,
//     Y: Player,
// {
//     fn bucket(&self) -> &B;
//     fn player(&self) -> &Y;
//     fn available(&self) -> Vec<&E>;
// }

// /// the thing that's able to self-replicate, defining how the tree builds itself
// trait Growable<E>
// where
//     Self: Sized,
//     E: Action,
// {
//     fn root() -> Self;
//     fn spawn(&self) -> Vec<(Self, E)>;
// }

// /// collection of these three is what you would get in a Node, which may be too restrictive for a lot of the use so we'll se
// trait Node<V, E, B, Y>
// where
//     Self: Visible<E, B, Y>,
//     V: Growable<E>,
//     E: Action,
//     B: Bucket,
//     Y: Player,
// {
//     // obserability
//     fn utility(&self, player: &Y) -> Utility;
//     // walkability
//     fn parent(&self) -> Option<&Self>;
//     fn children(&self) -> Vec<&Self>;
//     fn incoming(&self) -> Option<&E>;
//     fn outgoing(&self) -> Vec<&E>;
//     fn descendants(&self) -> Vec<&Self> {
//         match self.children().len() {
//             0 => vec![&self],
//             _ => self
//                 .children()
//                 .iter()
//                 .map(|child| child.descendants())
//                 .flatten()
//                 .collect(),
//         }
//     }
//     fn follow(&self, edge: &E) -> &Self {
//         self.children()
//             .iter()
//             .find(|child| edge == child.incoming().unwrap())
//             .unwrap()
//     }
// }

// /// implementation using petgraph::graph::DiGraph

// /// distribution over indistinguishable nodes, abstracted over the type of node
// trait Info<V, E, B, Y>
// where
//     Self: Visible<E, B, Y>,
//     E: Action,
//     B: Bucket,
//     Y: Player,
// {
//     fn roots(&self) -> Vec<&Node<V, E, B, Y>>;
// }

// /// a policy π is a distribution over actions given a node. Equivalently a vector indexed by action ∈ A
// trait Distribution<E>
// where
//     E: Action,
// {
//     fn weight(&self, action: &E) -> Probability;
//     fn sample(&self) -> &E;
// }

// /// a strategy σ is a policy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
// trait Strategy<D, E, B>
// where
//     D: Distribution<E>,
//     E: Action,
//     B: Bucket,
// {
//     fn policy(&self, bucket: &B) -> &D;
// }

// /// a profile σ consists of a strategy for each player. Equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
// trait Profile<S, D, N, E, B, Y>
// where
//     S: Strategy<D, E, B>,
//     N: Node<E, B, Y>,
//     D: Distribution<E>,
//     E: Action,
//     B: Bucket,
//     Y: Player,
// {
//     fn strategy(&self, player: &Y) -> &S;
//     // provided
//     fn gain(&self, root: &N, action: &E) -> Utility {
//         self.cfactual_value(root, action) - self.expected_value(root)
//     }
//     fn cfactual_value(&self, root: &N, action: &E) -> Utility {
//         self.cfactual_reach(root)
//             * root //                                       suppose you're here on purpose, counterfactually
//                 .follow(action) //                          suppose you're here on purpose, counterfactually
//                 .descendants() //                           O(depth) recursive downtree
//                 .iter() //                                  duplicated calculation
//                 .map(|leaf| self.relative_value(root, leaf))
//                 .sum::<Utility>()
//     }
//     fn expected_value(&self, root: &N) -> Utility {
//         self.expected_reach(root)
//             * root
//                 .descendants() //                           O(depth) recursive downtree
//                 .iter() //                                  duplicated calculation
//                 .map(|leaf| self.relative_value(root, leaf))
//                 .sum::<Utility>()
//     }
//     fn relative_value(&self, root: &N, leaf: &N) -> Utility {
//         leaf.utility(root.player())
//             * self.relative_reach(root, leaf)
//             * self.sampling_reach(root, leaf)
//     }
//     // probability calculations
//     fn weight(&self, node: &N, action: &E) -> Probability {
//         self.strategy(node.player())
//             .policy(node.bucket())
//             .weight(action)
//     }
//     fn cfactual_reach(&self, node: &N) -> Probability {
//         match node.parent() {
//             None => 1.0,
//             Some(parent) => {
//                 self.cfactual_reach(parent)
//                     * if node.player() == parent.player() {
//                         1.0
//                     } else {
//                         self.weight(
//                             parent,
//                             node.incoming().expect("if has parent, then has incoming"),
//                         )
//                     }
//             }
//         }
//     }
//     fn expected_reach(&self, node: &N) -> Probability {
//         match node.parent() {
//             None => 1.0,
//             Some(parent) => {
//                 self.expected_reach(parent)
//                     * self.weight(
//                         parent,
//                         node.incoming().expect("if has parent, then has incoming"),
//                     )
//             }
//         }
//     }
//     fn relative_reach(&self, root: &N, leaf: &N) -> Probability {
//         //? gotta optimize out integration over shared ancestors that cancels out in this division. Node: Eq? Hash?
//         self.expected_reach(leaf) / self.expected_reach(root)
//     }
//     fn sampling_reach(&self, _: &N, _: &N) -> Probability {
//         1.0
//     }
// }

// /// trainer will update regrets and profile in a mutable loop
// trait Trainer<P, S, D, N, E, B, I, Y>
// where
//     P: Profile<S, D, N, E, B, Y>,
//     S: Strategy<D, E, B>,
//     I: Info<N, E, B, Y>,
//     N: Node<E, B, Y>,
//     D: Distribution<E>,
//     E: Action,
//     B: Bucket,
//     Y: Player,
// {
//     fn profile(&self) -> &P;

//     fn update_regret(&mut self, info: &I);
//     fn update_policy(&mut self, info: &I);

//     fn current_regret(&self, info: &I, action: &E) -> Utility;
//     fn instant_regret(&self, info: &I, action: &E) -> Utility {
//         info.roots()
//             .iter()
//             .map(|root| self.profile().gain(root, action))
//             .sum::<Utility>()
//     }
//     fn pending_regret(&self, info: &I, action: &E) -> Utility {
//         self.instant_regret(info, action) + self.current_regret(info, action)
//     }

//     fn policy_vector(&self, info: &I) -> Vec<(E, Probability)> {
//         let regrets = info
//             .available()
//             .iter()
//             .map(|action| (**action, self.current_regret(info, action)))
//             .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
//             .collect::<Vec<(E, Probability)>>();
//         let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
//         let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
//         policy
//         // uses RegretMatching+ to compute policy from current regrets
//     }
//     fn regret_vector(&self, info: &I) -> Vec<(E, Utility)> {
//         info.available()
//             .iter()
//             .map(|action| (**action, self.pending_regret(info, action)))
//             .collect()
//     }
// }
