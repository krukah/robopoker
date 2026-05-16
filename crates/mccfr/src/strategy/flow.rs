use crate::*;
use rbp_core::*;
use rbp_transport::Density;
use std::collections::HashMap;

impl<T> CfrFlow for T where T: RefProf + CfrSampling {}

/// Counterfactual regret minimization math.
///
/// Blanket-implemented for all types implementing both [`Profile`] (read access
/// to accumulated regrets/weights) and [`CfrSampling`] (walker identity and sampling
/// parameters). Provides regret matching, reach probabilities, expected values,
/// and regret/policy vector computation.
///
/// This trait exists because the CFR math is a *gift* derived from having both
/// Profile and CfrSampling — it doesn't add required methods, so no one implements
/// it directly.
pub trait CfrFlow: RefProf + CfrSampling {
    /// Regret normalization constant for regret-matching denominator.
    fn regret_denom(&self, info: &Self::I) -> Utility {
        info.choices().map(|ref e| self.regret(info, e)).sum()
    }
    /// Sampling normalization constant including smoothing pseudocount.
    fn weight_denom(&self, info: &Self::I) -> Probability {
        info.choices()
            .map(|ref e| self.weight(info, e))
            .sum::<Probability>()
            + self.smoothing()
    }
    /// Compute sampling distribution for all edges of an info (single pass).
    /// Returns exploration-adjusted probabilities for MCCFR sampling.
    fn sampling_distribution(&self, info: &Self::I) -> Policy<Self::E> {
        let raw = info
            .choices()
            .map(|e| (e, self.weight(info, &e)))
            .collect::<Vec<_>>();
        let denom = raw.iter().map(|(_, w)| *w).sum::<Probability>() + self.smoothing();
        raw.into_iter()
            .map(|(e, w)| (e, w / self.temperature()))
            .map(|(e, w)| (e, w + self.smoothing()))
            .map(|(e, w)| (e, w / denom))
            .map(|(e, w)| (e, w.max(self.curiosity())))
            .collect()
    }
    /// Calculate immediate policy via regret matching for a single edge.
    /// Prefer `iterated_distribution` when multiple edges needed.
    fn instant_policy(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.regret(info, edge) / self.regret_denom(info)
    }
    /// Calculate sampling probability for a single edge.
    /// Prefer `sampling_distribution` when multiple edges needed.
    fn sampling(&self, info: &Self::I, edge: &Self::E) -> Probability {
        ((self.weight(info, edge) / self.temperature() + self.smoothing())
            / self.weight_denom(info))
        .max(self.curiosity())
    }

    /// Fused regret + expected value computation for an information set.
    /// Computes all action values once per root via DFS, then derives both
    /// regret and EV without redundant tree traversal.
    fn dfs(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> (Policy<Self::E>, Utility) {
        let span = infoset.span();
        let rd = self.regret_denom(infoset.head().info());
        let mut regrets = HashMap::<Self::E, Utility>::new();
        let mut payoff = 0.0;
        for root in &span {
            let reach = self.ancestor_reach(root);
            let actions = root
                .edges()
                .map(|(child, edge)| {
                    (
                        *edge,
                        reach * self.recursed_value(root, &root.at(child), 1.0, 1.0),
                    )
                })
                .collect::<Vec<_>>();
            let ev = actions
                .iter()
                .map(|(e, v)| self.regret(root.info(), e) / rd * v)
                .sum::<Utility>();
            payoff += ev;
            for (edge, cfv) in actions {
                debug_assert!(!cfv.is_nan());
                debug_assert!(!cfv.is_infinite());
                *regrets.entry(edge).or_default() += cfv - ev;
            }
        }
        (regrets.into_iter().collect(), payoff)
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
            .zip(expected)
            .flat_map(|(root, &ev)| {
                root.outgoing()
                    .into_iter()
                    .cloned()
                    .map(move |edge| (edge, self.gain(root, &edge, ev)))
            })
            .inspect(|(_, r)| debug_assert!(!r.is_nan()))
            .inspect(|(_, r)| debug_assert!(!r.is_infinite()))
            .fold(HashMap::new(), |mut acc, (edge, gain)| {
                *acc.entry(edge).or_default() += gain;
                acc
            })
            .into_iter()
            .collect()
    }
    /// Calculate immediate policy distribution from current regrets.
    ///
    /// Uses regret matching: pi(a) = max(regret(a), e) / sum max(regret, e).
    /// Actions with higher regret are chosen more frequently to minimize future regret.
    fn policy_vector(
        &self,
        infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Policy<Self::E> {
        self.iterated_distribution(&infoset.info())
    }
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
            .take_while(|a| &a.node() != root)
            .filter(|a| a.node().game().turn() != Self::T::chance())
            .map(|Ascent(incoming, parent)| self.instant_policy(parent.info(), &incoming))
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
        root.decisions()
            .filter(|(t, _, _)| *t != self.walker())
            .map(|(_, ref i, ref e)| self.instant_policy(i, e))
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
        leaf.decisions()
            .filter(|(t, _, _)| *t != self.walker())
            .map(|(_, ref i, ref e)| self.sampling(i, e))
            .product::<Probability>()
    }
    /// Constant factor for a root: cfactual_reach / sampling_above.
    /// Both products share the same path (root->tree_root) and filters
    /// (non-chance, non-walker), so we compute them in a single pass.
    fn ancestor_reach(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        let (cfactual, sampling) = root
            .decisions()
            .filter(|(t, _, _)| *t != self.walker())
            .fold((1.0, 1.0), |(cf, sm), (_, ref info, ref edge)| {
                (
                    cf * (self.regret(info, edge) / self.regret_denom(info)),
                    sm * ((self.weight(info, edge) / self.temperature() + self.smoothing())
                        / self.weight_denom(info))
                    .max(self.curiosity()),
                )
            });
        cfactual / sampling
    }

    /// Recursive DFS value computation. Accumulates reach during descent,
    /// eliminating descendants() allocation and per-leaf upward path walks.
    ///
    /// Computes: Σ_leaves [ payoff * pi_rel / pi_smp ]
    /// where pi_rel is accumulated relative reach (iterated for all non-chance)
    /// and pi_smp is accumulated sampling reach below root (sampling for non-chance, non-walker).
    fn recursed_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
        relative_reach: Probability,
        sampling_reach: Probability,
    ) -> Utility {
        if node.width() == 0 {
            return relative_reach / sampling_reach * self.terminal_value(node, root.game().turn());
        }
        let chance = node.game().turn() == Self::T::chance();
        let walker = node.game().turn() == self.walker();
        let regret_denom = (!chance).then(|| self.regret_denom(node.info()));
        let weight_denom = (!chance && !walker).then(|| self.weight_denom(node.info()));
        node.edges()
            .map(|(child, edge)| (node.at(child), edge))
            .map(|(ref child, edge)| {
                self.recursed_value(
                    root,
                    child,
                    relative_reach
                        * regret_denom.map_or(1.0, |d| self.regret(node.info(), edge) / d),
                    sampling_reach
                        * weight_denom.map_or(1.0, |d| {
                            ((self.weight(node.info(), edge) / self.temperature()
                                + self.smoothing())
                                / d)
                                .max(self.curiosity())
                        }),
                )
            })
            .sum()
    }
    /// Sum of relative values over a set of descendant leaves.
    fn ancestor_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaves: &[Node<Self::T, Self::E, Self::G, Self::I>],
    ) -> Utility {
        leaves
            .iter()
            .map(|leaf| self.relative_value(root, leaf))
            .sum::<Utility>()
    }
    /// Relative to the player at the root Node of this Infoset,
    /// what is the Utility contributed by this leaf Node?
    ///
    /// For terminal nodes, uses the game's payoff function.
    /// For frontier nodes (non-terminal leaves in depth-limited trees),
    /// uses the accumulated payoff from the profile.
    fn relative_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        leaf: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Utility {
        self.terminal_value(leaf, root.game().turn()) * self.relative_reach(root, leaf)
            / self.sampling_reach(leaf)
    }
    /// Policy-weighted expected utility at this node.
    ///
    /// V(I) = sum_a pi(a) * Q(I,a)
    ///
    /// Uses DFS subtree traversal instead of collecting descendants.
    fn expected_value(&self, root: &Node<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
        let weight = self.ancestor_reach(root);
        let policy = self.iterated_distribution(root.info());
        weight
            * root
                .edges()
                .map(|(i, e)| policy.density(e) * self.recursed_value(root, &root.at(i), 1.0, 1.0))
                .sum::<Utility>()
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    ///
    /// Uses DFS subtree traversal instead of collecting descendants.
    fn cfactual_value(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
    ) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
        root.step(edge)
            .map(|child| self.ancestor_reach(root) * self.recursed_value(root, &child, 1.0, 1.0))
            .expect("edge belongs to outgoing branches")
    }
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

    /// Using our current strategy Profile, how much regret
    /// would we gain by following this Edge at this Node?
    /// Takes pre-computed expected value to avoid redundant computation.
    fn gain(
        &self,
        root: &Node<Self::T, Self::E, Self::G, Self::I>,
        edge: &Self::E,
        baseline: Utility,
    ) -> Utility {
        debug_assert!(self.walker() == root.game().turn());
        self.cfactual_value(root, edge) - baseline
    }
    /// Deterministic RNG seeded by epoch, info set, and tree identity.
    /// Ensures the same edge is sampled for the same info set within
    /// a given epoch, while remaining unique across trees in a batch.
    fn rng(&self, node: &Node<Self::T, Self::E, Self::G, Self::I>) -> rand::rngs::SmallRng {
        use rand::SeedableRng;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;
        let ref mut hasher = DefaultHasher::new();
        self.t().hash(hasher);
        node.info().hash(hasher);
        node.seed().hash(hasher);
        rand::rngs::SmallRng::seed_from_u64(hasher.finish())
    }
}
