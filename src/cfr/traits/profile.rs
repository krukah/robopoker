use super::edge::Edge;
use super::game::Game;
use super::info::Info;
use super::turn::Turn;
use crate::cfr::structs::infoset::InfoSet;
use crate::cfr::structs::node::Node;
use crate::cfr::types::branch::Branch;
use crate::cfr::types::policy::Policy;

/// The `Profile` trait represents a strategy profile in an extensive-form game, implementing core
/// functionality for Counterfactual Regret Minimization (CFR).
///
/// A strategy profile maintains and updates:
/// - Accumulated regrets for each information set and action
/// - Accumulated weighted average strategies (policies) over time
/// - Current iteration/epoch tracking
///
/// # Key Concepts
///
/// ## Strategy Computation
/// - `policy_vector`: Computes immediate strategy distribution using regret-matching
/// - `policy`: Calculates immediate strategy from accumulated regrets
/// - `advice`: Provides long-run average strategy (Nash equilibrium approximation)
///
/// ## Reach Probabilities
/// - `expected_reach`: Probability of reaching a node following the current strategy
/// - `cfactual_reach`: Counterfactual reach probability (excluding player's own actions)
/// - `relative_reach`: Conditional probability of reaching a leaf from a given node
///
/// ## Utility and Regret
/// - `regret_vector`: Computes counterfactual regret for all actions in an information set
/// - `info_gain`: Immediate regret for not an action in an information set
/// - `node_gain`: Immediate regret for not an action at a specific node
///
/// ## Sampling
/// - `sample`: Implements various sampling schemes (e.g., external, targeted, uniform)
/// - `rng`: Provides deterministic random number generation for consistent sampling
///
/// # Implementation Notes
///
/// Implementors must provide:
/// - `increment`: Update epoch/iteration counter
/// - `walker`: Current player's turn
/// - `epochs`: Number of iterations completed
/// - `weight`: Access to accumulated action weights/policies
/// - `regret`: Access to accumulated regrets
/// - `sample`: Custom sampling strategy
///
/// The trait provides automatic implementations for strategy computation, reach probabilities,
/// and utility calculations based on these core methods.
pub trait Profile {
    type T: Turn;
    type E: Edge;
    type G: Game<E = Self::E, T = Self::T>;
    type I: Info<E = Self::E, T = Self::T>;

    /// increment epoch
    fn increment(&mut self);
    /// who's turn is it?
    fn walker(&self) -> Self::T;
    /// how many iterations
    fn epochs(&self) -> usize;
    /// lookup accumulated policy for this information
    fn weight(&self, info: &Self::I, edge: &Self::E) -> crate::Probability;
    /// lookup accumulated regret for this information
    fn regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility;
    /// topology-based sampling. i.e. external, probing, targeted, uniform, etc.
    fn sample(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<Branch<Self::E, Self::G>>,
    ) -> Vec<Branch<Self::E, Self::G>>;

    /// automatic

    /// Using our current strategy Profile,
    /// compute the regret vector
    /// by calculating the marginal Utitlity
    /// missed out on for not having followed
    /// every walkable Edge at this Infoset/Node/Bucket
    fn regret_vector(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Policy<Self::E> {
        let regrets = infoset
            .info()
            .choices()
            .into_iter()
            .map(|e| (e, self.info_gain(infoset, &e)))
            .inspect(|(_, r)| assert!(!r.is_nan()))
            .inspect(|(_, r)| assert!(!r.is_infinite()))
            .collect::<Policy<Self::E>>();
        regrets
    }
    /// calculate immediate policy distribution from current regrets, ignoring historical weighted policies.
    /// this uses regret matching, which converts regret values into probabilities by:
    /// 1. taking the positive portion of each regret (max with 0)
    /// 2. normalizing these values to sum to 1.0 to form a valid probability distribution
    /// this ensures actions with higher regret are chosen more frequently to minimize future regret.
    fn policy_vector(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Policy<Self::E> {
        let info = infoset.info();
        let regrets = info
            .choices()
            .into_iter()
            .map(|e| (e, self.regret(&info, &e)))
            .map(|(a, r)| (a, r.max(crate::POLICY_MIN)))
            .collect::<Policy<Self::E>>();
        let denominator = regrets
            .iter()
            .map(|(_, r)| r)
            .inspect(|r| assert!(**r >= 0.))
            .sum::<crate::Utility>();
        let policy = regrets
            .into_iter()
            .map(|(a, r)| (a, r / denominator))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect::<Policy<Self::E>>();
        policy
    }

    // strategy calculations

    /// calculate immediate weighted average decision
    /// strategy for this information.
    /// i.e. policy from accumulated REGRET values
    fn policy(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.regret(info, edge).max(crate::POLICY_MIN)
            / info
                .choices()
                .iter()
                .map(|e| self.regret(info, e))
                .inspect(|r| assert!(!r.is_nan()))
                .inspect(|r| assert!(!r.is_infinite()))
                .map(|r| r.max(crate::POLICY_MIN))
                .sum::<crate::Utility>()
    }
    /// calculate the long-run weighted average decision
    /// strategy for this information.
    /// i.e. policy from accumulated POLICY values
    fn advice(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.weight(info, edge).max(crate::POLICY_MIN)
            / info
                .choices()
                .iter()
                .map(|e| self.weight(info, e))
                .inspect(|r| assert!(!r.is_nan()))
                .inspect(|r| assert!(!r.is_infinite()))
                .inspect(|r| assert!(*r >= 0.))
                .map(|r| r.max(crate::POLICY_MIN))
                .sum::<crate::Probability>()
    }

    // reach calculations

    /// at the immediate location of this Node,
    /// what is the Probability of transitioning via this Edge?
    fn outgoing_reach(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> crate::Probability {
        self.policy(node.info(), edge)
    }
    /// Conditional on being in a given Infoset,
    /// what is the Probability of
    /// visiting this particular leaf Node,
    /// assuming we all follow the distribution offered by Profile?
    fn relative_reach(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Probability {
        leaf.into_iter()
            .take_while(|(parent, _)| parent != root)
            .map(|(parent, incoming)| self.outgoing_reach(&parent, &incoming))
            .product::<crate::Probability>()
    }
    /// If we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn expected_reach(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Probability {
        root.into_iter()
            .map(|(parent, incoming)| self.outgoing_reach(&parent, &incoming))
            .product::<crate::Probability>()
    }
    /// If, counterfactually, we had played toward this infoset,
    /// then what would be the Probability of us being in this infoset?
    /// i.e. assuming our opponents played according to distributions from Profile, but we did not.
    ///
    /// This function also serves as a form of importance sampling.
    /// MCCFR requires we adjust our reach in counterfactual
    /// regret calculation to account for the under- and over-sampling
    /// of regret across different Infosets.
    fn cfactual_reach(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Probability {
        root.into_iter()
            .filter(|(parent, _)| self.walker() != parent.game().turn())
            .map(|(parent, incoming)| self.outgoing_reach(&parent, &incoming))
            .product::<crate::Probability>()
    }
    /// In Monte Carlo CFR variants, we sample actions according to some
    /// sampling strategy q(a) (possibly in place of the current policy p(a)).
    /// To correct for this bias, we multiply regrets by p(a)/q(a).
    /// This function returns q(a), the probability that we sampled
    /// the actions leading to this node under our sampling scheme.
    /// For vanilla CFR, q(a) = 1.0 since we explore all actions.
    #[allow(unused)]
    fn sampling_reach(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Probability {
        1.0
    }

    // utility calculations

    /// relative to the player at the root Node of this Infoset,
    /// what is the Utility of this leaf Node?
    fn relative_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Utility {
        self.relative_reach(root, leaf) * leaf.game().payoff(root.game().turn())
    }
    /// Assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon
    /// visiting this Node?
    fn expected_value(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        self.expected_reach(root) / self.sampling_reach(root)
            * root
                .descendants()
                .iter()
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<crate::Utility>()
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        self.cfactual_reach(root) / self.sampling_reach(root)
            * root
                .follow(edge)
                .expect("edge belongs to outgoing branches")
                .descendants()
                .iter()
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<crate::Utility>()
    }

    // counterfactual gain calculations

    /// Conditional on being in this Infoset,
    /// distributed across all its head Nodes,
    /// with paths weighted according to our Profile:
    /// if we follow this Edge 100% of the time,
    /// what is the expected marginal increase in Utility?
    fn info_gain(
        &self,
        info: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> crate::Utility {
        info.span()
            .iter()
            .map(|root| self.node_gain(root, edge))
            .inspect(|r| assert!(!r.is_nan()))
            .inspect(|r| assert!(!r.is_infinite()))
            .sum::<crate::Utility>()
    }
    /// Using our current strategy Profile, how much regret
    /// would we gain by following this Edge at this Node?
    fn node_gain(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        let cfactual = self.cfactual_value(root, edge);
        let expected = self.expected_value(root);
        cfactual - expected
    }

    // deterministic sampling

    /// deterministically sampling the same Edge for the same Infoset
    /// requries decision-making to be Info-level
    fn rng(&self, info: &Self::I) -> rand::rngs::SmallRng {
        use rand::SeedableRng;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        let ref mut hasher = DefaultHasher::new();
        self.epochs().hash(hasher);
        info.hash(hasher);
        rand::rngs::SmallRng::seed_from_u64(hasher.finish())
    }
}
