use super::*;
use crate::mccfr::*;
use crate::transport::Density;
use crate::*;

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
/// - `policy`: Access to accumulated action weights/policies
/// - `regret`: Access to accumulated regrets
///
/// The trait provides automatic implementations for strategy computation, reach probabilities,
/// and utility calculations based on these core methods.
pub trait Profile {
    type T: TreeTurn;
    type E: TreeEdge;
    type G: TreeGame<E = Self::E, T = Self::T>;
    type I: TreeInfo<E = Self::E, T = Self::T>;

    // unimplemented

    /// increment epoch
    fn increment(&mut self);
    /// who's turn is it?
    fn walker(&self) -> Self::T;
    /// how many iterations
    fn epochs(&self) -> usize;
    /// lookup accumulated policy for this information
    fn sum_policy(&self, info: &Self::I, edge: &Self::E) -> Probability;
    /// lookup accumulated regret for this information
    fn sum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility;
    /// optional metrics for logging (default: None)
    fn metrics(&self) -> Option<&Metrics> {
        None
    }

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
        vec![choices.remove(rng.random_range(0..n))]
    }
    /// Profile-weighted sampling using accumulated average policy values.
    fn explore_one(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<Branch<Self::E, Self::G>>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        use rand::distr::Distribution;
        use rand::distr::weighted::WeightedIndex;
        let ref info = node.info();
        let ref mut rng = self.rng(info);
        let ref samples = self.sampling_distribution(info);
        let mut choices = branches;
        let weights = choices
            .iter()
            .map(|(edge, _, _)| samples.density(edge))
            .map(|weight| weight.max(POLICY_MIN))
            .collect::<Vec<_>>();
        vec![
            choices.remove(
                WeightedIndex::new(weights)
                    .expect("at least one policy > 0")
                    .sample(rng),
            ),
        ]
    }

    // update vector calculations

    /// Compute regret gains for all edges. Pre-computes expected values
    /// for all roots to avoid redundant computation.
    fn regret_vector(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Policy<Self::E> {
        let ref span = infoset.span();
        let ref expected = span
            .iter()
            .map(|r| self.expected_value(r))
            .collect::<Vec<_>>();
        infoset
            .info()
            .choices()
            .into_iter()
            .map(|edge| {
                let gain = span
                    .iter()
                    .zip(expected.iter())
                    .map(|(root, &ev)| self.node_gain(root, &edge, ev))
                    .inspect(|r| assert!(!r.is_nan()))
                    .inspect(|r| assert!(!r.is_infinite()))
                    .sum::<Utility>();
                (edge, gain)
            })
            .collect()
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
            .map(|e| (e, self.sum_regret(&info, &e)))
            .map(|(a, r)| (a, r.max(POLICY_MIN)))
            .collect::<Policy<Self::E>>();
        let denominator = regrets
            .iter()
            .map(|(_, r)| r)
            .inspect(|r| assert!(**r >= 0.))
            .sum::<Utility>();
        let policy = regrets
            .into_iter()
            .map(|(a, r)| (a, r / denominator))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect::<Policy<Self::E>>();
        policy
    }

    // batch strategy calculations (single pass per info)

    /// Compute policy distribution for all edges of an info (single pass).
    /// Returns full distribution using regret-matching.
    fn matching_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let denom = info
            .choices()
            .iter()
            .map(|e| self.sum_regret(info, e))
            .inspect(|r| assert!(!r.is_nan()))
            .inspect(|r| assert!(!r.is_infinite()))
            .map(|r| r.max(POLICY_MIN))
            .sum::<Utility>();
        info.choices()
            .into_iter()
            .map(|e| (e, self.sum_regret(info, &e)))
            .map(|(e, r)| (e, r.max(POLICY_MIN)))
            .map(|(e, r)| (e, r / denom))
            .collect()
    }
    /// Compute sampling distribution for all edges of an info (single pass).
    /// Returns exploration-adjusted probabilities for MCCFR sampling.
    fn sampling_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let denom = info
            .choices()
            .iter()
            .map(|e| self.sum_policy(info, e))
            .inspect(|r| assert!(!r.is_nan()))
            .inspect(|r| assert!(!r.is_infinite()))
            .map(|r| r.max(POLICY_MIN))
            .sum::<Probability>()
            + self.activation();
        info.choices()
            .into_iter()
            .map(|e| (e, self.sum_policy(info, &e)))
            .map(|(e, p)| (e, p.max(POLICY_MIN)))
            .map(|(e, p)| (e, p * self.threshold()))
            .map(|(e, p)| (e, p + self.activation()))
            .map(|(e, p)| (e, p / denom))
            .map(|(e, p)| (e, p.max(self.exploration())))
            .collect()
    }
    /// Compute advice distribution for all edges of an info (single pass).
    /// Returns historical weighted average strategy (Nash approximation).
    fn averaged_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let denom = info
            .choices()
            .iter()
            .map(|e| self.sum_policy(info, e))
            .inspect(|r| assert!(!r.is_nan()))
            .inspect(|r| assert!(!r.is_infinite()))
            .map(|r| r.max(POLICY_MIN))
            .sum::<Probability>();
        info.choices()
            .into_iter()
            .map(|e| (e, self.sum_policy(info, &e)))
            .map(|(e, p)| (e, p.max(POLICY_MIN)))
            .map(|(e, p)| (e, p / denom))
            .collect()
    }

    // per-edge strategy calculations (convenience wrappers)

    /// Calculate immediate policy via regret matching for a single edge.
    /// Prefer `policy_distribution` when multiple edges needed.
    fn matching(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.matching_distribution(info).density(edge)
    }
    /// Calculate historical average for a single edge.
    /// Prefer `advice_distribution` when multiple edges needed.
    fn averaged(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.averaged_distribution(info).density(edge)
    }
    /// Calculate sampling probability for a single edge.
    /// Prefer `sample_distribution` when multiple edges needed.
    fn sampling(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.sampling_distribution(info).density(edge)
    }

    // reach calculations
    // chance nodes are filtered because:
    // 1. with external sampling, exactly 1 chance branch is taken (probability = 1)
    // 2. chance terms would cancel in relative_value = reach / sampling anyway
    // 3. filtering avoids unnecessary computation for Edge::Draw

    /// Conditional on being in a given Infoset,
    /// what is the Probability of
    /// visiting this particular leaf Node,
    /// assuming we all follow the distribution offered by Profile?
    fn relative_reach(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Probability {
        leaf.into_iter()
            .take_while(|(parent, _)| parent != root)
            .filter(|(parent, _)| parent.game().turn() != Self::T::chance())
            .map(|(parent, incoming)| self.matching(parent.info(), &incoming))
            .product::<Probability>()
    }
    /// If we were to play by the Profile,
    /// up to this Node in the Tree,
    /// then what is the probability of visiting this Node?
    fn expected_reach(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> Probability {
        root.into_iter()
            .filter(|(parent, _)| parent.game().turn() != Self::T::chance())
            .map(|(parent, incoming)| self.matching(parent.info(), &incoming))
            .product::<Probability>()
    }
    /// If, counterfactually, we had played toward this infoset,
    /// then what would be the Probability of us being in this infoset?
    /// i.e. assuming our opponents played according to distributions from Profile, but we did not.
    ///
    /// This function also serves as a form of importance sampling.
    /// MCCFR requires we adjust our reach in counterfactual
    /// regret calculation to account for the under- and over-sampling
    /// of regret across different Infosets.
    fn cfactual_reach(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> Probability {
        root.into_iter()
            .filter(|(parent, _)| parent.game().turn() != Self::T::chance())
            .filter(|(parent, _)| parent.game().turn() != self.walker())
            .map(|(parent, incoming)| self.matching(parent.info(), &incoming))
            .product::<Probability>()
    }
    /// In Monte Carlo CFR variants, we sample actions according to some
    /// sampling strategy q(a) (possibly in place of the current policy p(a)).
    /// To correct for this bias, we multiply regrets by p(a)/q(a).
    /// This function returns q(a), the probability that we sampled
    /// the actions leading to this node under our sampling scheme.
    ///
    /// For vanilla CFR, q(a) = 1.0 since we explore all actions.
    fn sampling_reach(&self, leaf: &Node<Self::T, Self::E, Self::G, Self::I>) -> Probability {
        leaf.into_iter()
            .filter(|(parent, _)| parent.game().turn() != Self::T::chance())
            .filter(|(parent, _)| parent.game().turn() != self.walker())
            .map(|(parent, incoming)| self.sampling(parent.info(), &incoming))
            .product::<Probability>()
    }

    // utility calculations

    /// Sum of relative values over a set of descendant leaves.
    fn ancestor_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        kids: &[Node<Self::T, Self::E, Self::G, Self::I>],
    ) -> Utility {
        kids.iter()
            .map(|leaf| self.relative_value(root, leaf))
            .sum::<Utility>()
    }
    /// Relative to the player at the root Node of this Infoset,
    /// what is the Utility contributed by this leaf Node?
    fn relative_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Utility {
        self.relative_reach(root, leaf) * leaf.game().payoff(root.game().turn())
            / self.sampling_reach(leaf)
    }
    /// Assuming we start at root Node,
    /// and that we sample the Tree according to Profile,
    /// how much Utility do we expect upon visiting this Node?
    fn expected_value(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        assert!(self.walker() == root.game().turn());
        let ref descendants = root.descendants();
        self.ancestor_value(root, descendants) * self.expected_reach(root)
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> Utility {
        assert!(self.walker() == root.game().turn());
        let ref descendants = root
            .follow(edge)
            .expect("edge belongs to outgoing branches")
            .descendants();
        self.ancestor_value(root, descendants) * self.cfactual_reach(root)
    }

    // counterfactual gain calculations

    /// Using our current strategy Profile, how much regret
    /// would we gain by following this Edge at this Node?
    /// Takes pre-computed expected value to avoid redundant computation.
    fn node_gain(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
        expected: Utility,
    ) -> Utility {
        assert!(self.walker() == root.game().turn());
        self.cfactual_value(root, edge) - expected
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
    fn threshold(&self) -> Entropy {
        SAMPLING_THRESHOLD
    }
    /// Beta (β) - inertia parameter that stabilizes strategy updates by weighting
    /// historical policies. Set to 0.5 to balance between stability and adaptiveness.
    fn activation(&self) -> Energy {
        SAMPLING_ACTIVATION
    }
    /// Epsilon (ε) - exploration parameter that ensures minimum sampling probability
    /// for each action to maintain exploration. Set to 0.01 based on empirical testing
    /// which showed better convergence compared to higher values.
    fn exploration(&self) -> Probability {
        SAMPLING_EXPLORATION
    }
}
