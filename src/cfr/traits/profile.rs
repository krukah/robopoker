use crate::cfr::structs::infoset::InfoSet;
use crate::cfr::structs::node::Node;
use crate::cfr::traits::edge::Edge;
use crate::cfr::traits::game::Game;
use crate::cfr::traits::info::Info;
use crate::cfr::traits::turn::Turn;
use crate::cfr::types::branch::Branch;
use crate::cfr::types::policy::Policy;

/// the strategy is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to determine:
///  what is the Density over the Edges
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

    /// calculate immediate weighted average decision
    /// strategy for this information.
    /// i.e. policy from accumulated REGRET values
    fn policy(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.regret(info, edge).max(crate::POLICY_MIN)
            / info
                .choices()
                .iter()
                .map(|e| self.regret(info, e))
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
                .map(|r| r.max(crate::POLICY_MIN))
                .sum::<crate::Probability>()
    }

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
            .map(|edge| (edge, self.info_gain(infoset, &edge)))
            .inspect(|(_, r)| assert!(!r.is_nan()))
            .inspect(|(_, r)| assert!(!r.is_infinite()))
            .map(|(e, r)| (e, r.max(crate::REGRET_MIN)))
            .collect::<Policy<Self::E>>();
        log::info!("regret vector @ {:?}: {:?}", infoset.info(), regrets);
        regrets
    }
    /// calculate immediate policy distribution from current regrets, ignoring historical weighted policies
    fn policy_vector(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Policy<Self::E> {
        let regrets = infoset
            .info()
            .choices()
            .into_iter()
            .map(|e| (e, self.regret(&infoset.info(), &e)))
            .map(|(a, r)| (a, r.max(crate::POLICY_MIN)))
            .collect::<Policy<Self::E>>();
        let denominator = regrets
            .iter()
            .map(|(_, r)| r)
            .inspect(|r| assert!(**r > 0.))
            .sum::<crate::Utility>();
        let policy = regrets
            .into_iter()
            .map(|(a, r)| (a, r / denominator))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect::<Policy<Self::E>>();
        log::info!("policy vector @ {:?}: {:?}", infoset.info(), policy);
        policy
    }

    /// at the immediate location of this Node,
    /// what is the Probability of transitioning via this Edge?
    fn outgoing_reach(
        &self,
        node: Node<Self::T, Self::E, Self::G, Self::I>,
        edge: Self::E,
    ) -> crate::Probability {
        self.policy(&node.info(), &edge)
    }
    /// Conditional on being in a given Infoset,
    /// what is the Probability of
    /// visiting this particular leaf Node,
    /// given the distribution offered by Profile?
    fn relative_reach(
        &self,
        root: Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Probability {
        if root.index() == leaf.index() {
            1.0
        } else {
            match leaf.up() {
                None => unreachable!("leaf must be downstream of root"),
                Some((up, edge)) => self.relative_reach(root, up) * self.outgoing_reach(up, *edge),
            }
        }
    }
    /// If we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn expected_reach(&self, root: Node<Self::T, Self::E, Self::G, Self::I>) -> crate::Probability {
        match root.up() {
            None => 1.0,
            Some((up, edge)) => self.expected_reach(up) * self.outgoing_reach(up, *edge),
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
    fn cfactual_reach(&self, node: Node<Self::T, Self::E, Self::G, Self::I>) -> crate::Probability {
        match node.up() {
            None => 1.0,
            Some((up, edge)) => {
                if self.walker() != up.game().turn() {
                    self.cfactual_reach(up) * self.outgoing_reach(up, *edge)
                } else {
                    self.cfactual_reach(up)
                }
            }
        }
    }

    /// relative to the player at the root Node of this Infoset,
    /// what is the Utility of this leaf Node?
    fn relative_value(
        &self,
        root: Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Utility {
        leaf.game().payoff(root.game().turn())
    }
    /// Assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon
    /// visiting this Node?
    fn expected_value(&self, root: Node<Self::T, Self::E, Self::G, Self::I>) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        self.expected_reach(root)
            * root
                .descendants()
                .into_iter()
                .map(|leaf| self.relative_value(root, leaf) * self.relative_reach(root, leaf))
                .sum::<crate::Utility>()
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(
        &self,
        root: Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        root.follow(edge)
            .expect("edge belongs to outgoing")
            .descendants()
            .into_iter()
            .map(|leaf| self.relative_value(root, leaf) * self.expected_reach(leaf))
            .sum::<crate::Utility>()
            / self.cfactual_reach(root)
    }

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
            .into_iter()
            .inspect(|root| assert!(self.walker() == root.game().turn()))
            .map(|root| self.node_gain(root, edge))
            .inspect(|r| assert!(!r.is_nan()))
            .inspect(|r| assert!(!r.is_infinite()))
            .sum::<crate::Utility>()
    }
    /// Using our current strategy Profile, how much regret
    /// would we gain by following this Edge at this Node?
    fn node_gain(
        &self,
        root: Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        let cfactual = self.cfactual_value(root, edge);
        let expected = self.expected_value(root);
        cfactual - expected
    }

    /// deterministically sampling the same Edge for the same Infoset
    /// requries decision-making to be Info-level
    fn rng(&self, info: &Self::I) -> rand::rngs::SmallRng {
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
