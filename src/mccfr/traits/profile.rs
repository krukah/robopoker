use super::edge::Edge;
use super::game::Game;
use super::info::Info;
use super::turn::Turn;
use crate::mccfr::structs::infoset::InfoSet;
use crate::mccfr::structs::node::Node;
use crate::mccfr::types::branch::Branch;
use crate::mccfr::types::policy::Policy;

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

    // unimplemented

    /// increment epoch
    fn increment(&mut self);
    /// who's turn is it?
    fn walker(&self) -> Self::T;
    /// how many iterations
    fn epochs(&self) -> usize;
    /// lookup accumulated policy for this information
    fn sum_policy(&self, info: &Self::I, edge: &Self::E) -> crate::Probability;
    /// lookup accumulated regret for this information
    fn sum_regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility;

    // exploration calculations

    /// topology-based sampling. i.e. external, probing, targeted, uniform, etc.
    ///
    /// this default implementation is opinionated about using
    /// external average discounted sampling
    /// - external: only the current traverser's actions are fully explored
    /// - average: the accumulated policy values are used to weight samples
    /// - discounted: a discounting schedule can adapt sensitiviy
    ///
    /// For vanilla CFR, no-op because we sample all actions
    fn explore(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<Branch<Self::E, Self::G>>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        let n = branches.len();
        let p = node.game().turn();
        let walker = self.walker();
        let chance = Self::T::chance();
        match (n, p) {
            (0, _) => branches,
            (_, p) if p == walker => branches,
            (_, p) if p == chance => self.explore_any(node, branches),
            (_, p) if p != walker => self.explore_one(node, branches),
            _ => panic!("at the disco"),
        }
    }
    /// uniform sampling of available branches
    fn explore_any(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<Branch<Self::E, Self::G>>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        use rand::Rng;
        assert!(!branches.is_empty());
        let n = branches.len();
        let mut choices = branches;
        let ref mut rng = self.rng(node.info());
        let choice = rng.random_range(0..n);
        let chosen = choices.remove(choice);
        vec![chosen]
    }
    /// profile-weighted by ACCUMULATED WEIGHTED AVERAGE
    /// policy policy values. discounting is encapsulated
    /// by self.discount(_)
    fn explore_one(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<Branch<Self::E, Self::G>>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        use rand::distr::weighted::WeightedIndex;
        use rand::distr::Distribution;
        let ref info = node.info();
        let ref mut rng = self.rng(info);
        let mut choices = branches;
        // Fused pass to compute sampling weights without recomputing denominators per edge.
        // q(a) = max(exploration, (activation + threshold * weight(a)) / (activation + sum(weights)))
        // where weight(a) = accumulated policy for (info, a)
        let weights = choices
            .iter()
            .map(|(edge, _, _)| self.sum_policy(info, edge).max(crate::POLICY_MIN))
            .collect::<Vec<_>>();
        let denom = self.activation() + weights.iter().copied().sum::<crate::Probability>();
        let policy = weights
            .iter()
            .map(|&w| ((self.activation() + w * self.threshold()) / denom).max(self.exploration()))
            .collect::<Vec<_>>();
        let choice = WeightedIndex::new(policy)
            .expect("at least one policy > 0")
            .sample(rng);
        let chosen = choices.remove(choice);
        vec![chosen]
    }

    // update vector calculations

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
        // Fused pass: compute positive regrets and denominator in one loop, then normalize.
        let mut tmp: Policy<Self::E> = Vec::with_capacity(info.choices().len());
        let mut denom: crate::Utility = 0.0;
        for edge in info.choices().into_iter() {
            let r = self.sum_regret(&info, &edge).max(crate::POLICY_MIN);
            denom += r;
            tmp.push((edge, r));
        }
        assert!(denom > 0.0);
        tmp.into_iter()
            .map(|(a, r)| (a, r / denom))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect::<Policy<Self::E>>()
    }

    // strategy calculations

    /// calculate IMMEDIATE weighted average decision
    /// strategy for this information.
    /// i.e. policy from accumulated REGRET values
    fn policy(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        // Single-pass denominator accumulation for clarity; per-edge API limits reuse across calls.
        let numer = self.sum_regret(info, edge).max(crate::POLICY_MIN);
        let mut denom: crate::Utility = 0.0;
        for e in info.choices().iter() {
            let r = self.sum_regret(info, e).max(crate::POLICY_MIN);
            assert!(!r.is_nan() && !r.is_infinite());
            denom += r;
        }
        numer / denom
    }
    /// calculate the HISTORICAL WEIGHTED AVERAGE decision
    /// strategy for this information.
    /// i.e. policy from accumulated POLICY values
    fn advice(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        let numer = self.sum_policy(info, edge).max(crate::POLICY_MIN);
        let mut denom: crate::Probability = 0.0;
        for e in info.choices().iter() {
            let w = self.sum_policy(info, e).max(crate::POLICY_MIN);
            assert!(!w.is_nan() && !w.is_infinite() && w >= 0.0);
            denom += w;
        }
        numer / denom
    }
    /// In Monte Carlo CFR variants, we sample actions according to a sampling strategy q(a).
    /// This function computes q(a) for a given action in an infoset, which is used for importance sampling.
    /// The sampling probability is based on the action weights, temperature, inertia, and exploration parameters.
    /// The formula is: q(a) = max(exploration, (inertia + temperature * weight(a)) / (inertia + sum(weights)))
    fn sample(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        // Keep per-edge API but avoid extra iterator adapters; accumulate denom directly.
        let numer = self.sum_policy(info, edge).max(crate::POLICY_MIN);
        let mut denom: crate::Probability = 0.0;
        for e in info.choices().iter() {
            let w = self.sum_policy(info, e).max(crate::POLICY_MIN);
            assert!(!w.is_nan() && !w.is_infinite() && w >= 0.0);
            denom += w;
        }
        let denom = self.activation() + denom;
        let numer = self.activation() + numer * self.threshold();
        (numer / denom).max(self.exploration())
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
            .take_while(|(parent, _)| parent != root) // parent.index() > root.index()
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
    ///
    /// For vanilla CFR, q(a) = 1.0 since we explore all actions.
    fn sampling_reach(
        &self,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> crate::Probability {
        leaf.into_iter()
            .filter(|(parent, _)| self.walker() != parent.game().turn())
            .map(|(parent, incoming)| self.sample(parent.info(), &incoming))
            .product::<crate::Probability>()
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
            / self.sampling_reach(leaf) // importance sampling
                                        //
                                        // this could be sped up by not recalculating
                                        // the "sampling_reach" of root redundantly,
                                        // since it contributes the same multiplicative
                                        // reach probability factor to the sampling_reach
                                        // of each leaf. instead, sampling_reach could get
                                        // absorbed into relative_reach, while the common
                                        // sampling_reach(root) distributes.
                                        //
                                        // i.e. sampling_reach(Node, Option<Node>) , where
                                        // None indicates go back to tree root.
    }
    /// Assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon
    /// visiting this Node?
    fn expected_value(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> crate::Utility {
        assert!(self.walker() == root.game().turn());
        self.expected_reach(root)
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
        self.cfactual_reach(root)
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

    // constant fns

    /// Tau (τ) - temperature parameter that controls sampling greediness.
    /// Set to 0.5 to make sampling more focused on promising actions while
    /// still maintaining some exploration.
    fn threshold(&self) -> crate::Entropy {
        crate::SAMPLING_THRESHOLD
    }
    /// Beta (β) - inertia parameter that stabilizes strategy updates by weighting
    /// historical policies. Set to 0.5 to balance between stability and adaptiveness.
    fn activation(&self) -> crate::Energy {
        crate::SAMPLING_ACTIVATION
    }
    /// Epsilon (ε) - exploration parameter that ensures minimum sampling probability
    /// for each action to maintain exploration. Set to 0.01 based on empirical testing
    /// which showed better convergence compared to higher values.
    fn exploration(&self) -> crate::Probability {
        crate::SAMPLING_EXPLORATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccfr::rps::edge::Edge;
    use crate::mccfr::rps::game::Game;
    use crate::mccfr::rps::turn::Turn;
    use crate::mccfr::structs::infoset::InfoSet;
    use crate::mccfr::structs::tree::Tree;
    use crate::mccfr::traits::game::Game as GameTrait;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_infoset(turn: Turn) -> InfoSet<Turn, Edge, Game, Turn> {
        let mut tree: Tree<Turn, Edge, Game, Turn> = Tree::default();
        let index = {
            let node = tree.seed(turn, <Game as GameTrait>::root());
            node.index()
        };
        let mut infoset = InfoSet::from(Arc::new(tree));
        infoset.push(index);
        infoset
    }

    #[derive(Default)]
    struct TestProfile {
        epochs: usize,
        policies: HashMap<Turn, HashMap<Edge, f32>>, // accumulated policy weights
        regrets: HashMap<Turn, HashMap<Edge, f32>>,  // accumulated regrets
    }

    impl Profile for TestProfile {
        type T = Turn;
        type E = Edge;
        type G = Game;
        type I = Turn; // Info == Turn for RPS

        fn increment(&mut self) {
            self.epochs += 1;
        }
        fn walker(&self) -> Self::T {
            Turn::P1
        }
        fn epochs(&self) -> usize {
            self.epochs
        }
        fn sum_policy(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
            self.policies
                .get(info)
                .and_then(|m| m.get(edge))
                .copied()
                .unwrap_or_default()
        }
        fn sum_regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility {
            self.regrets
                .get(info)
                .and_then(|m| m.get(edge))
                .copied()
                .unwrap_or_default()
        }
    }

    #[test]
    fn policy_vector_normalizes_and_matches_regret_matching() {
        // Prepare profile with specific regrets for Turn::P1
        let mut profile = TestProfile::default();
        profile
            .regrets
            .entry(Turn::P1)
            .or_default()
            .extend([(Edge::R, 1.0), (Edge::P, 3.0), (Edge::S, 0.0)]);

        let infoset = make_infoset(Turn::P1);

        // Compute policy vector
        let policy = Profile::policy_vector(&profile, &infoset);
        // Expected via regret matching with floor at POLICY_MIN
        let r_r = profile.sum_regret(&Turn::P1, &Edge::R).max(crate::POLICY_MIN);
        let r_p = profile.sum_regret(&Turn::P1, &Edge::P).max(crate::POLICY_MIN);
        let r_s = profile.sum_regret(&Turn::P1, &Edge::S).max(crate::POLICY_MIN);
        let denom = r_r + r_p + r_s;
        let exp_r = r_r / denom;
        let exp_p = r_p / denom;
        let exp_s = r_s / denom;

        let get = |e: Edge| policy.iter().find(|(a, _)| *a == e).unwrap().1;
        let pr = get(Edge::R);
        let pp = get(Edge::P);
        let ps = get(Edge::S);

        assert!((pr - exp_r).abs() < 1e-6);
        assert!((pp - exp_p).abs() < 1e-6);
        assert!((ps - exp_s).abs() < 1e-6);
        let sum: f32 = pr + pp + ps;
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sampling_q_matches_formula() {
        // Prepare profile with specific accumulated policy weights
        let mut profile = TestProfile::default();
        profile
            .policies
            .entry(Turn::P1)
            .or_default()
            .extend([(Edge::R, 0.10), (Edge::P, 0.30), (Edge::S, 0.60)]);

        // Expected q(a) per formula
        let w_r = profile.sum_policy(&Turn::P1, &Edge::R).max(crate::POLICY_MIN);
        let w_p = profile.sum_policy(&Turn::P1, &Edge::P).max(crate::POLICY_MIN);
        let w_s = profile.sum_policy(&Turn::P1, &Edge::S).max(crate::POLICY_MIN);
        let denom = profile.activation() + (w_r + w_p + w_s);
        let q = |w: f32| ((profile.activation() + w * profile.threshold()) / denom)
            .max(profile.exploration());

        let qr = Profile::sample(&profile, &Turn::P1, &Edge::R);
        let qp = Profile::sample(&profile, &Turn::P1, &Edge::P);
        let qs = Profile::sample(&profile, &Turn::P1, &Edge::S);

        assert!((qr - q(w_r)).abs() < 1e-6);
        assert!((qp - q(w_p)).abs() < 1e-6);
        assert!((qs - q(w_s)).abs() < 1e-6);
    }
}
