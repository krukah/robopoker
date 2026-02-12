use crate::*;
use rbp_core::*;
use rbp_transport::Density;
use std::collections::BTreeMap;
use std::collections::HashMap;

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
/// - `cfactual_reach`: Counterfactual reach probability (excluding player's own actions)
/// - `relative_reach`: Conditional probability of reaching a leaf from a given node
/// - `sampling_reach`: Sampling probability for importance correction
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
pub trait Profile: Sized {
    type T: CfrTurn;
    type E: CfrEdge;
    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T>;
    // unimplemented

    /// increment epoch
    fn increment(&mut self);
    /// who's turn is it?
    fn walker(&self) -> Self::T;
    /// how many iterations
    fn epochs(&self) -> usize;
    /// lookup accumulated weight for this information
    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability;
    /// lookup accumulated regret for this information
    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility;
    /// lookup accumulated expected value for this information-action pair
    fn cum_evalue(&self, info: &Self::I, edge: &Self::E) -> Utility;
    /// lookup accumulated encounter counts for this information-action pair
    fn cum_counts(&self, info: &Self::I, edge: &Self::E) -> u32;

    /// optional metrics for logging (default: None)
    fn metrics(&self) -> Option<&Metrics> {
        None
    }
    /// Compute the average expected value at an information set.
    ///
    /// Returns the weight-averaged EV across all actions. This is used
    /// for frontier evaluation in depth-limited search and safe subgame solving.
    ///
    /// The computation is: sum(weight[a] * ev[a]) / sum(weight[a]) for all actions a.
    fn frontier_evalue(&self, info: &Self::I) -> Utility {
        let choices = info.choices();
        let denom = choices
            .iter()
            .map(|e| self.cum_weight(info, e))
            .map(|p| p.max(POLICY_MIN))
            .sum::<Probability>();
        choices
            .into_iter()
            .map(|e| self.cum_weight(info, &e).max(POLICY_MIN) * self.cum_evalue(info, &e))
            .sum::<Utility>()
            / denom
    }

    // update vector calculations

    /// Compute the expected value of an information set under current strategy.
    ///
    /// This is the sum of expected values over all nodes in the infoset span.
    /// Used for EV accumulation during training and frontier evaluation.
    fn infoset_value(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        infoset
            .span()
            .iter()
            .map(|r| self.expected_value(r))
            .sum::<Utility>()
    }

    /// Compute regret gains for all edges. Pre-computes expected values
    /// for all roots to avoid redundant computation.
    ///
    /// Iterates per-node over each node's actual outgoing edges, since
    /// sampling may have expanded different edges at different nodes.
    fn regret_vector(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Policy<Self::E> {
        let ref span = infoset.span();
        let ref expected = span
            .iter()
            .map(|r| self.expected_value(r))
            .collect::<Vec<_>>();
        span.iter()
            .zip(expected.iter())
            .flat_map(|(root, &evalue)| {
                root.outgoing()
                    .into_iter()
                    .cloned()
                    .map(move |edge| (edge, self.node_gain(root, &edge, evalue)))
            })
            .inspect(|(_, r)| debug_assert!(!r.is_nan()))
            .inspect(|(_, r)| debug_assert!(!r.is_infinite()))
            .fold(
                std::collections::HashMap::<Self::E, Utility>::new(),
                |mut acc, (edge, gain)| {
                    *acc.entry(edge).or_default() += gain;
                    acc
                },
            )
            .into_iter()
            .collect()
    }
    /// Calculate immediate policy distribution from current regrets.
    ///
    /// Uses regret matching: π(a) = max(regret(a), ε) / Σ max(regret, ε).
    /// Actions with higher regret are chosen more frequently to minimize future regret.
    fn policy_vector(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Policy<Self::E> {
        self.iterated_distribution(&infoset.info())
    }

    // batch strategy calculations (single pass per info)

    /// Compute policy distribution for all edges of an info (single pass).
    /// Returns full distribution using regret-matching.
    fn iterated_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let denom = info
            .choices()
            .iter()
            .map(|e| self.cum_regret(info, e))
            .inspect(|r| debug_assert!(!r.is_nan()))
            .inspect(|r| debug_assert!(!r.is_infinite()))
            .map(|r| r.max(POLICY_MIN))
            .sum::<Utility>();
        info.choices()
            .into_iter()
            .map(|e| (e, self.cum_regret(info, &e)))
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
            .map(|e| self.cum_weight(info, e))
            .inspect(|r| debug_assert!(!r.is_nan()))
            .inspect(|r| debug_assert!(!r.is_infinite()))
            .map(|r| r.max(POLICY_MIN))
            .sum::<Probability>()
            + self.smoothing();
        info.choices()
            .into_iter()
            .map(|e| (e, self.cum_weight(info, &e)))
            .map(|(e, p)| (e, p.max(POLICY_MIN)))
            .map(|(e, p)| (e, p / self.temperature()))
            .map(|(e, p)| (e, p + self.smoothing()))
            .map(|(e, p)| (e, p / denom))
            .map(|(e, p)| (e, p.max(self.curiosity())))
            .collect()
    }
    /// Compute advice distribution for all edges of an info (single pass).
    /// Returns historical weighted average strategy (Nash approximation).
    fn averaged_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let denom = info
            .choices()
            .iter()
            .map(|e| self.cum_weight(info, e))
            .inspect(|r| debug_assert!(!r.is_nan()))
            .inspect(|r| debug_assert!(!r.is_infinite()))
            .map(|r| r.max(POLICY_MIN))
            .sum::<Probability>();
        info.choices()
            .into_iter()
            .map(|e| (e, self.cum_weight(info, &e)))
            .map(|(e, p)| (e, p.max(POLICY_MIN)))
            .map(|(e, p)| (e, p / denom))
            .collect()
    }

    // per-edge strategy calculations (convenience wrappers)

    /// Calculate immediate policy via regret matching for a single edge.
    /// Prefer `policy_distribution` when multiple edges needed.
    fn iterated(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.iterated_distribution(info).density(edge)
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
            .map(|(parent, ref incoming)| self.iterated(parent.info(), incoming))
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
            .map(|(parent, ref incoming)| self.iterated(parent.info(), incoming))
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
            .map(|(parent, ref incoming)| self.sampling(parent.info(), incoming))
            .product::<Probability>()
    }
    /// Product of external (opponent) strategy probabilities along path to node.
    ///
    /// Iterates **upward** from node to root via `node.into_iter()`, filtering
    /// to external decision points and multiplying averaged probabilities.
    fn external_reach(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        hero: Self::T,
    ) -> Probability {
        node.into_iter()
            .filter(|(p, _)| p.game().turn() != Self::T::chance()) // exclude chance
            .filter(|(p, _)| p.game().turn() != hero) // exclude ourselves (is this always self.walker() ?)
            .map(|(p, e)| self.averaged(p.info(), &e))
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
    ///
    /// For terminal nodes, uses the game's payoff function.
    /// For frontier nodes (non-terminal leaves in depth-limited trees),
    /// uses the accumulated expected value from the profile.
    fn relative_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Utility {
        (if leaf.game().turn() == Self::T::terminal() {
            leaf.game().payoff(root.game().turn())
        } else {
            self.frontier_evalue(leaf.info())
        }) * self.relative_reach(root, leaf)
            / self.sampling_reach(leaf)
    }
    /// Policy-weighted expected utility at this node.
    ///
    /// V(I) = Σ_a π(a) × Q(I,a)
    ///
    /// The state value equals the policy-weighted sum of action values,
    /// ensuring regret(a) = Q(a) - V(I) depends on the action's own value.
    fn expected_value(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
        root.outgoing()
            .iter()
            .map(|edge| self.iterated(root.info(), edge) * self.cfactual_value(root, edge))
            .sum()
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
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
        debug_assert!(self.walker() == root.game().turn());
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

    /// Temperature (T) - controls sampling entropy via policy scaling.
    /// σ'(a) = max(ε, (σ(a)/T + β) / (Σσ + β))
    /// Higher T → more uniform (exploratory); lower T → more peaked (greedy).
    fn temperature(&self) -> Entropy {
        SAMPLING_TEMPERATURE
    }
    /// Smoothing (β) - pseudocount added to numerator and denominator.
    /// σ'(a) = max(ε, (σ(a)/T + β) / (Σσ + β))
    /// Higher values pull sampling toward uniform (maximum entropy prior).
    fn smoothing(&self) -> Energy {
        SAMPLING_SMOOTHING
    }
    /// Epsilon (ε) - minimum sampling probability floor.
    /// σ'(a) = max(ε, (τ·σ(a) + β) / (Σσ + β))
    /// Ensures every action retains at least ε probability for exploration.
    fn curiosity(&self) -> Probability {
        SAMPLING_CURIOSITY
    }

    // exploitability and best response

    /// Computes the exploitability of the current average strategy.
    ///
    /// Exploitability measures how far the strategy is from Nash equilibrium.
    /// For a two-player zero-sum game:
    ///
    /// `exploitability = (BR(P1) + BR(P2)) / 2`
    ///
    /// where `BR(Pi)` is the expected utility that player i can achieve by
    /// playing a best response against the opponent's fixed average strategy.
    ///
    /// A Nash equilibrium has exploitability of 0. Lower values indicate
    /// strategies closer to equilibrium.
    ///
    /// # Implementation
    /// Exploitability = (BR(P1) + BR(P2)) / 2 where BR is best response value.
    fn exploitability(&self, tree: Tree<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        let ref partition = tree.partition();
        0.5 * (0.
            + self.optimal_response_evalue(partition, Self::T::from(0))
            + self.optimal_response_evalue(partition, Self::T::from(1)))
    }

    /// Expected value at external (opponent-controlled) subtree.
    ///
    /// **Recursive descent** through tree children, weighting by opponent's
    /// averaged strategy at each external node. Handles terminal, frontier,
    /// and chance nodes. Contrast with upward iteration in [`Self::external_reach`].
    fn external_evalue(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        hero: Self::T,
    ) -> Utility {
        self.subgamed_evalue(node, hero, None)
    }
    /// Recursive expected value using precomputed best response actions.
    fn response_evalue(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        hero: Self::T,
        br: &BTreeMap<Self::I, Self::E>,
    ) -> Utility {
        self.subgamed_evalue(node, hero, Some(br))
    }
    /// Recursive expected value computation.
    /// When `br` is `None`, computes external-only EV (panics on hero nodes).
    /// When `br` is `Some`, computes best-response EV (hero follows BR actions).
    fn subgamed_evalue(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        hero: Self::T,
        br: Option<&BTreeMap<Self::I, Self::E>>,
    ) -> Utility {
        let n = node.width();
        let kids = node.children();
        let recurse = |x| self.subgamed_evalue(x, hero, br);
        match node.game().turn() {
            t if t == Self::T::terminal() => node.game().payoff(hero),
            _ if n == 0 => self.frontier_evalue(node.info()),
            t if t == Self::T::chance() => kids.iter().map(recurse).sum::<Utility>() / n as Utility,
            t if t == hero => br
                .map(|br| br.get(node.info()).expect("edge seen in BR"))
                .map(|e| node.follow(e).expect("hero unreachable without BR"))
                .as_ref()
                .map(recurse)
                .expect("BR is available Edge"),
            _ => kids
                .iter()
                .map(|x| self.averaged(node.info(), x.incoming().expect("non-root")) * recurse(x))
                .sum(),
        }
    }

    /// Best response value: optimal play for `hero` against opponents' average strategy.
    /// Respects info set structure by choosing one action per info set, not per node.
    fn optimal_response_evalue(
        &self,
        partition: &HashMap<Self::I, InfoSet<Self::T, Self::E, Self::G, Self::I>>,
        hero: Self::T,
    ) -> Utility {
        let ref root = partition
            .values()
            .next()
            .expect("partition")
            .tree()
            .at(petgraph::graph::NodeIndex::new(0));
        let ref response = partition
            .iter()
            .filter(|(_, infoset)| infoset.head().game().turn() == hero)
            .map(|(info, infoset)| (info.clone(), self.optimal_cfactual_choice(infoset, hero)))
            .collect::<BTreeMap<_, _>>();
        self.response_evalue(root, hero, response)
    }

    /// Counterfactual value of an edge in an info set.
    fn optimal_cfactual_evalue(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
        hero: &Self::T,
    ) -> Utility {
        infoset
            .span()
            .iter()
            .filter_map(|n| n.follow(edge))
            .map(|c| self.external_reach(&c, *hero) * self.external_evalue(&c, *hero))
            .sum()
    }

    /// Best action at an info set: argmax over actions of counterfactual value.
    fn optimal_cfactual_choice(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
        hero: Self::T,
    ) -> Self::E {
        let ref evalues = infoset
            .info()
            .choices()
            .into_iter()
            .map(|edge| (edge, self.optimal_cfactual_evalue(&infoset, &edge, &hero)))
            .collect::<std::collections::HashMap<_, _>>();
        infoset
            .info()
            .choices()
            .into_iter()
            .max_by(|a, b| {
                let a = evalues.get(a).expect("computed evalue for action a");
                let b = evalues.get(b).expect("computed evalue for action b");
                a.partial_cmp(b).expect("good values")
            })
            .expect("info set has actions")
    }
}
