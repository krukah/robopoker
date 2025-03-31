use super::branch::Branch;
use super::edge::Edge;
use super::game::Game;
use super::info::Info;
use super::infoset::InfoSet;
use super::node::Node;
use super::policy::Policy;
use super::turn::Turn;

/// the strategy is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to determine:
///  what is the Density over the Edges
pub trait Profile<T, E, G, I>
where
    T: Turn,
    E: Edge,
    G: Game<E = E, T = T>,
    I: Info<E = E, T = T>,
{
    /// who's turn is it?
    fn walker(&self) -> T;
    /// how many iterations
    fn epochs(&self) -> usize;
    /// lookup historical policy distribution, given this information
    fn policy(&self, info: &I) -> &Policy<E>;
    /// lookup historical regret value, given this information
    fn regret(&self, info: &I) -> &Policy<E>;
    /// topology-based sampling. i.e. external, probing, targeted, uniform, etc.
    fn sample(&self, node: &Node<T, E, G, I>, branches: Vec<Branch<E, G>>) -> Vec<Branch<E, G>> ;
   
   
   ///
   
    // {
    //     let n = branches.len();
    //     let p = node.game().turn();
    //     let chance = todo!();
    //     let walker = self.walker();
    //     match (n, p) {
    //         (0, _) => branches,
    //         (_, p) if p == walker => branches,
    //         (_, p) if p == chance => self.sample_any(node, branches),
    //         (_, p) if p != walker => self.sample_one(node, branches),
    //         _ => panic!("at the disco"),
    //     }
    // }
    fn sample_any(&self, node: &Node<T, E, G, I>, edges: Vec<Branch<E, G>>) -> Vec<Branch<E, G>> {
        use rand::Rng;
        let n = edges.len();
        let mut edges = edges;
        let ref mut rng = self.rng(node.info());
        let choice = rng.gen_range(0..n);
        let chosen = edges.remove(choice);
        vec![chosen]
    }
    fn sample_one(&self, node: &Node<T, E, G, I>, edges: Vec<Branch<E, G>>) -> Vec<Branch<E, G>> {
        use crate::transport::density::Density;
        use rand::distributions::WeightedIndex;
        use rand::prelude::Distribution;
        let info = node.info();
        let ref mut rng = self.rng(info);
        let mut choices = edges;
        let policy = choices
            .iter()
            .map(|(edge, _, _)| self.policy(info).density(edge))
            .collect::<Vec<crate::Probability>>();
        let choice = WeightedIndex::new(policy)
            .expect("at least one policy > 0")
            .sample(rng);
        let chosen = choices.remove(choice);
        vec![chosen]
    }

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

    /// relative to the player at the root Node of this Infoset,
    /// what is the Utility of this leaf Node?
    fn relative_value(&self, root: Node<T, E, G, I>, leaf: Node<T, E, G, I>) -> crate::Utility {
        leaf.game().payoff(root.game().turn())
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
                .map(|leaf| {
                    1.0
                    * self.relative_value(root, leaf) //.
                    * self.relative_reach(root, leaf) //.
                })
                .sum::<crate::Utility>()
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(&self, root: Node<T, E, G, I>, edge: &E) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        self.cfactual_reach(root)
            * root
                .follow(edge)
                .expect("edge belongs to outgoing")
                .descendants()
                .into_iter()
                .map(|leaf| {
                    1.0  
                    * self.relative_value(root, leaf) //.
                    * self.relative_reach(root, leaf) //.
                    / self.cfactual_reach(leaf) //.
                })
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
        assert!(self.walker() == root.game().turn());
        self.cfactual_value(root, edge) - self.expected_value(root)
    }

    /// deterministically sampling the same Edge for the same Infoset
    /// requries decision-making to be Info-level
    fn rng(&self, info: &I) -> rand::rngs::SmallRng {
        use rand::SeedableRng;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        let ref mut hasher = DefaultHasher::new();
        info.hash(hasher);
        self.epochs().hash(hasher);
        rand::rngs::SmallRng::seed_from_u64(hasher.finish())
    }
}
