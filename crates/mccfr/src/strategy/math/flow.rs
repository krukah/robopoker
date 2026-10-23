use crate::*;
use monge::Density;
use pokerkit::*;

impl<T> Flow for T where T: Profile + Clock {}

/// Counterfactual regret minimization math: regret matching, reach
/// probabilities, expected values, regret/policy vectors.
///
/// Blanket-implemented, never implemented directly: the math is a *gift* derived
/// from having both [`Profile`] (accumulated regrets/weights) and [`Clock`]
/// (walker identity, sampling parameters), so it adds no required methods.
pub trait Flow: Profile + Clock {
    /// Regret normalization constant for regret-matching denominator.
    fn regret_denom(&self, info: &InfoOf<Self>) -> Utility {
        info.menu().map(|ref e| self.regret(info, e)).sum::<Utility>()
    }
    /// Sampling normalization constant including smoothing pseudocount.
    fn weight_denom(&self, info: &InfoOf<Self>) -> Probability {
        info.menu().map(|ref e| self.weight(info, e)).sum::<Probability>() + self.smoothing()
    }
    /// Unnormalized exploration weight for one edge: `max(ε, (σ(a)/τ + β) / (Σσ + β))`.
    /// `denom` is [`Self::weight_denom`], hoisted so full-infoset callers compute it once.
    /// Sums to `Z ≥ 1` over an infoset; divide by that sum to get a probability.
    fn sampling_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>, denom: Probability) -> Probability {
        ((self.weight(info, edge) / self.temperature() + self.smoothing()) / denom).max(self.curiosity())
    }
    /// `(denom, Z)`: [`Self::weight_denom`] and the infoset's sum of sampling weights, so any
    /// edge's sampling probability is `sampling_weight(info, edge, denom) / Z`.
    fn sampling_denoms(&self, info: &InfoOf<Self>) -> (Probability, Probability) {
        let denom = self.weight_denom(info);
        let z = info
            .menu()
            .map(|ref e| self.sampling_weight(info, e, denom))
            .sum::<Probability>();
        (denom, z)
    }
    /// Normalized distribution over an infoset's edges that the sampler draws from.
    fn sampling_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        let (denom, z) = self.sampling_denoms(info);
        info.menu()
            .map(|e| (e, self.sampling_weight(info, &e, denom) / z))
            .collect()
    }
    /// Calculate immediate policy via regret matching for a single edge.
    /// Prefer `iterated_strategy` when multiple edges needed.
    fn instant_policy(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.regret(info, edge) / self.regret_denom(info)
    }
    /// Normalized probability the external sampler draws `edge` — exactly what
    /// `WeightedIndex` samples from, so the correct importance term for reaches.
    /// Prefer `sampling_strategy` when multiple edges needed.
    fn sampling(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        let (denom, z) = self.sampling_denoms(info);
        self.sampling_weight(info, edge, denom) / z
    }

    /// Fused regret + expected value for an infoset: one DFS per root yields the
    /// action values from which both are derived, avoiding a second traversal.
    fn dfs(&self, infoset: &InfoSet<Self::F>) -> (Over<EdgeOf<Self>, Regret>, Utility) {
        let span = infoset.span();
        let rd = self.regret_denom(infoset.head().info());
        let mut regrets = FxHashMap::<EdgeOf<Self>, Utility>::default();
        let mut payoff = 0.0;
        for root in &span {
            let reach = self.ancestor_reach(root);
            let actions = root
                .edges()
                .map(|(child, edge)| (*edge, reach * self.recursed_value(root, &root.at(child), 1.0, 1.0)))
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

    /// Regret gains for all edges, with root expected values hoisted.
    ///
    /// Iterates per-node over each node's *actual* outgoing edges, since
    /// sampling may have expanded different edges at different nodes.
    fn regret_vector(&self, infoset: &InfoSet<Self::F>) -> Over<EdgeOf<Self>, Regret> {
        let ref span = infoset.span();
        let expected = &span.iter().map(|r| self.expected_value(r)).collect::<Vec<_>>();
        span.iter()
            .zip(expected)
            .flat_map(|(root, &ev)| {
                root.outgoing()
                    .into_iter()
                    .copied()
                    .map(move |edge| (edge, self.gain(root, &edge, ev)))
            })
            .inspect(|(_, r)| debug_assert!(!r.is_nan()))
            .inspect(|(_, r)| debug_assert!(!r.is_infinite()))
            .fold(FxHashMap::default(), |mut acc, (edge, gain)| {
                *acc.entry(edge).or_default() += gain;
                acc
            })
            .into_iter()
            .collect()
    }
    /// Immediate policy from current regrets by regret matching:
    /// `pi(a) = max(regret(a), e) / sum max(regret, e)`.
    fn policy_vector(&self, infoset: &InfoSet<Self::F>) -> Over<EdgeOf<Self>, Policy> {
        self.iterated_strategy(&infoset.info())
    }
    /// Reach of `leaf` from `root` when every player follows the profile.
    fn relative_reach(&self, root: &Node<Self::F>, leaf: &Node<Self::F>) -> Probability {
        leaf.into_iter()
            .take_while(|a| &a.node() != root)
            .filter(|a| a.node().game().turn() != <TurnOf<Self>>::chance())
            .map(|Ascent(incoming, parent)| self.instant_policy(parent.info(), &incoming))
            .product::<Probability>()
    }
    /// If, counterfactually, we had played toward this infoset, then what would be the
    /// Probability of us being in this infoset? i.e. assuming our opponents played according to
    /// distributions from Profile, but we did not.
    ///
    /// This function also serves as a form of importance sampling.
    fn cfactual_reach(&self, root: &Node<Self::F>) -> Probability {
        root.decisions()
            .filter(|choice| choice.turn() != self.walker())
            .map(|choice| self.instant_policy(choice.info(), choice.edge()))
            .product::<Probability>()
    }
    /// In Monte Carlo CFR variants, we sample actions according to some sampling strategy q(a)
    /// (possibly in place of the current policy p(a)). To correct for this bias, we multiply
    /// regrets by p(a)/q(a). This function returns q(a), the probability that we sampled the
    /// actions leading to this node under our sampling scheme.
    ///
    /// For vanilla CFR, q(a) = 1.0 since we explore all actions.
    fn sampling_reach(&self, leaf: &Node<Self::F>) -> Probability {
        leaf.decisions()
            .filter(|choice| choice.turn() != self.walker())
            .map(|choice| self.sampling(choice.info(), choice.edge()))
            .product::<Probability>()
    }
    /// Constant factor for a root: cfactual_reach / sampling_above.
    /// Both products share the same path (root->tree_root) and filters
    /// (non-chance, non-walker), so we compute them in a single pass.
    fn ancestor_reach(&self, root: &Node<Self::F>) -> Utility {
        let (cfactual, sampling) =
            root.decisions()
                .filter(|choice| choice.turn() != self.walker())
                .fold((1.0, 1.0), |(cf, sm), choice| {
                    (
                        cf * self.instant_policy(choice.info(), choice.edge()),
                        sm * self.sampling(choice.info(), choice.edge()),
                    )
                });
        cfactual / sampling
    }

    /// `Σ_leaves [ payoff * pi_rel / pi_smp ]`, where `pi_rel` is relative reach
    /// (all non-chance) and `pi_smp` is sampling reach below root (non-chance,
    /// non-walker).
    ///
    /// Reach accumulates during descent, avoiding a `descendants()` allocation
    /// and a per-leaf upward path walk.
    fn recursed_value(
        &self,
        root: &Node<Self::F>,
        node: &Node<Self::F>,
        relative_reach: Probability,
        sampling_reach: Probability,
    ) -> Utility {
        if node.width() == 0 {
            return relative_reach / sampling_reach * self.terminal_value(node, root.game().turn());
        }
        let chance = node.game().turn() == <TurnOf<Self>>::chance();
        let walker = node.game().turn() == self.walker();
        let regret_denom = (!chance).then(|| self.regret_denom(node.info()));
        let sampling = (!chance && !walker).then(|| self.sampling_denoms(node.info()));
        node.edges()
            .map(|(child, edge)| (node.at(child), edge))
            .map(|(ref child, edge)| {
                self.recursed_value(
                    root,
                    child,
                    relative_reach * regret_denom.map_or(1.0, |d| self.regret(node.info(), edge) / d),
                    sampling_reach
                        * sampling.map_or(1.0, |(denom, z)| self.sampling_weight(node.info(), edge, denom) / z),
                )
            })
            .sum()
    }
    fn ancestor_value(&self, root: &Node<Self::F>, leaves: &[Node<Self::F>]) -> Utility {
        leaves
            .iter()
            .map(|leaf| self.relative_value(root, leaf))
            .sum::<Utility>()
    }
    /// Relative to the player at the root Node of this Infoset,
    /// what is the Utility contributed by this leaf Node?
    ///
    /// Terminal nodes use the game's payoff; frontier nodes (non-terminal leaves
    /// in depth-limited trees) use the profile's accumulated payoff.
    fn relative_value(&self, root: &Node<Self::F>, leaf: &Node<Self::F>) -> Utility {
        self.terminal_value(leaf, root.game().turn()) * self.relative_reach(root, leaf) / self.sampling_reach(leaf)
    }
    /// Policy-weighted expected utility at this node: `V(I) = Σ_a pi(a) * Q(I,a)`.
    fn expected_value(&self, root: &Node<Self::F>) -> Utility {
        debug_assert_eq!(self.walker(), root.game().turn());
        let weight = self.ancestor_reach(root);
        let policy = self.iterated_strategy(root.info());
        weight
            * root
                .edges()
                .map(|(i, e)| policy.density(e) * self.recursed_value(root, &root.at(i), 1.0, 1.0))
                .sum::<Utility>()
    }
    /// If, counterfactually,
    /// we had intended to get ourselves in this infoset,
    /// then what would be the expected Utility of this leaf?
    fn cfactual_value(&self, root: &Node<Self::F>, edge: &EdgeOf<Self>) -> Utility {
        debug_assert_eq!(self.walker(), root.game().turn());
        root.step(edge)
            .map(|child| self.ancestor_reach(root) * self.recursed_value(root, &child, 1.0, 1.0))
            .expect("edge belongs to outgoing branches")
    }
    /// Expected value of an infoset: summed over every node in its span.
    fn infoset_value(&self, infoset: &InfoSet<Self::F>) -> Utility {
        infoset.span().iter().map(|r| self.expected_value(r)).sum::<Utility>()
    }

    /// Using our current strategy Profile, how much regret
    /// would we gain by following this Edge at this Node?
    /// Takes pre-computed expected value to avoid redundant computation.
    fn gain(&self, root: &Node<Self::F>, edge: &EdgeOf<Self>, baseline: Utility) -> Utility {
        debug_assert_eq!(self.walker(), root.game().turn());
        self.cfactual_value(root, edge) - baseline
    }
    /// Deterministic RNG seeded by epoch, info set, and tree identity.
    /// Ensures the same edge is sampled for the same info set within
    /// a given epoch, while remaining unique across trees in a batch.
    fn rng(&self, node: &Node<Self::F>) -> rand::rngs::SmallRng {
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
