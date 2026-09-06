use crate::*;
use monge::Density;
use pokerkit::*;

/// Async variant of [`CfrSolution`] for database-backed or parallel training.
///
/// CFR is identical whether data lives in memory or a database — only access
/// differs. So in-memory profiles get this trait free via the blanket impl below,
/// while DB-backed workers implement it directly with real async I/O.
///
/// Methods that would collide with the sync traits are renamed: `traverser()` for
/// `walker()`, `iteration()` for `epochs()`.
#[async_trait::async_trait]
pub trait AsyncProfile: Send + Sync {
    type T: CfrTurn + Send + Sync;
    type E: CfrEdge + Send + Sync;
    type G: CfrGame<E = Self::E, T = Self::T> + Send + Sync;
    type I: CfrInfo<E = Self::E, T = Self::T> + Send + Sync;
    /// Current traversing player.
    fn traverser(&self) -> Self::T;
    /// Current training iteration.
    fn iteration(&self) -> usize;
    fn temperature(&self) -> Entropy {
        SamplingHyperParams::get().temperature()
    }
    fn smoothing(&self) -> Energy {
        SamplingHyperParams::get().smoothing()
    }
    /// Exploration floor.
    fn curiosity(&self) -> Probability {
        SamplingHyperParams::get().curiosity()
    }
    /// Current iteration strategy via regret matching.
    async fn policy(&self, info: &Self::I) -> Policy<Self::E>;
    /// Exploration-adjusted sampling distribution.
    async fn sample(&self, info: &Self::I) -> Policy<Self::E>;
    /// Historical average strategy (Nash approximation).
    async fn advice(&self, info: &Self::I) -> Policy<Self::E>;
    /// Fused cfactual/sampling reach in one upward pass, batch-fetching
    /// policies and samples via `join_all` before folding both products.
    async fn ancestor_reach(&self, root: &Node<'_, Self::T, Self::E, Self::G, Self::I>) -> Utility {
        let path = root
            .decisions()
            .filter(|(t, _, _)| *t != self.traverser())
            .map(|(_, i, e)| (i, e))
            .collect::<Vec<_>>();
        let (policies, samples) = futures::future::join(
            futures::future::join_all(path.iter().map(|(i, _)| self.policy(i))),
            futures::future::join_all(path.iter().map(|(i, _)| self.sample(i))),
        )
        .await;
        let (cf, sm) = path
            .iter()
            .zip(policies.iter())
            .zip(samples.iter())
            .fold((1.0, 1.0), |(cf, sm), (((_, e), pol), smp)| (cf * pol.density(e), sm * smp.density(e)));
        cf / sm
    }
    /// Recursive DFS accumulating reach during descent. Fetches policy (+ sample
    /// if non-walker) once per internal node, then `.density(edge)` per child.
    async fn recursed_value(
        &self,
        root: &Node<'_, Self::T, Self::E, Self::G, Self::I>,
        node: &Node<'_, Self::T, Self::E, Self::G, Self::I>,
        rel: Probability,
        smp: Probability,
    ) -> Utility {
        if node.width() == 0 {
            return rel / smp * node.game().payoff(root.game().turn());
        }
        let chance = node.game().turn() == Self::T::chance();
        let walker = node.game().turn() == self.traverser();
        let policy = if chance { None } else { Some(self.policy(node.info()).await) };
        let sample = if !chance && !walker { Some(self.sample(node.info()).await) } else { None };
        let mut total = 0.0;
        for (child, edge) in node.edges() {
            let r = rel * policy.as_ref().map_or(1.0, |p| p.density(edge));
            let s = smp * sample.as_ref().map_or(1.0, |q| q.density(edge));
            total += self.recursed_value(root, &node.at(child), r, s).await;
        }
        total
    }
    /// Fused regret + EV per infoset: one `ancestor_reach` and one policy fetch
    /// per root, then `recursed_value` per edge — no second tree traversal.
    async fn dfs(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> (Policy<Self::E>, Utility) {
        let span = infoset.span();
        let mut regrets = std::collections::HashMap::<Self::E, Utility>::new();
        let mut payoff = 0.0;
        for root in &span {
            let reach = self.ancestor_reach(root).await;
            let policy = self.policy(root.info()).await;
            let mut actions = Vec::new();
            for (child, edge) in root.edges() {
                let v = reach * self.recursed_value(root, &root.at(child), 1.0, 1.0).await;
                actions.push((*edge, v));
            }
            let ev = actions.iter().map(|(e, v)| policy.density(e) * v).sum::<Utility>();
            payoff += ev;
            for (edge, cfv) in actions {
                debug_assert!(!cfv.is_nan());
                debug_assert!(!cfv.is_infinite());
                *regrets.entry(edge).or_default() += cfv - ev;
            }
        }
        (regrets.into_iter().collect(), payoff)
    }
    /// Policy-weighted expected utility at this node.
    async fn expected_value(&self, root: &Node<'_, Self::T, Self::E, Self::G, Self::I>) -> Utility {
        debug_assert_eq!(self.traverser(), root.game().turn());
        let reach = self.ancestor_reach(root).await;
        let policy = self.policy(root.info()).await;
        let mut total = 0.0;
        for (child, edge) in root.edges() {
            total += policy.density(edge) * self.recursed_value(root, &root.at(child), 1.0, 1.0).await;
        }
        reach * total
    }
    /// Counterfactual value of taking an action at a node.
    async fn cfactual_value(&self, root: &Node<'_, Self::T, Self::E, Self::G, Self::I>, edge: &Self::E) -> Utility {
        debug_assert_eq!(self.traverser(), root.game().turn());
        let reach = self.ancestor_reach(root).await;
        let child = root.step(edge).expect("edge belongs to outgoing branches");
        reach * self.recursed_value(root, &child, 1.0, 1.0).await
    }
    /// Expected value of an information set under current strategy.
    async fn infoset_value(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        self.dfs(infoset).await.1
    }
    /// Regret gains for all edges in an information set.
    async fn regret_vector(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Policy<Self::E> {
        self.dfs(infoset).await.0
    }
    async fn policy_vector(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Policy<Self::E> {
        self.policy(&infoset.info()).await
    }
}

/// Blanket [`AsyncProfile`] for all [`CfrSolution`] types: every method delegates
/// to its synchronous counterpart, so in-memory profiles work in async contexts.
#[async_trait::async_trait]
impl<P> AsyncProfile for P
where
    P: CfrSolution + Send + Sync,
    P::T: Send + Sync,
    P::E: Send + Sync,
    P::G: Send + Sync,
    P::I: Send + Sync,
{
    type T = P::T;
    type E = P::E;
    type G = P::G;
    type I = P::I;

    fn traverser(&self) -> Self::T {
        CfrSampling::walker(self)
    }

    fn iteration(&self) -> usize {
        RefProf::t(self)
    }

    fn temperature(&self) -> Entropy {
        CfrSampling::temperature(self)
    }

    fn smoothing(&self) -> Energy {
        CfrSampling::smoothing(self)
    }

    fn curiosity(&self) -> Probability {
        CfrSampling::curiosity(self)
    }

    async fn policy(&self, info: &Self::I) -> Policy<Self::E> {
        self.iterated_distribution(info)
    }

    async fn sample(&self, info: &Self::I) -> Policy<Self::E> {
        self.sampling_distribution(info)
    }

    async fn advice(&self, info: &Self::I) -> Policy<Self::E> {
        self.averaged_distribution(info)
    }

    async fn ancestor_reach(&self, root: &Node<'_, Self::T, Self::E, Self::G, Self::I>) -> Utility {
        CfrFlow::ancestor_reach(self, root)
    }

    async fn recursed_value(
        &self,
        root: &Node<'_, Self::T, Self::E, Self::G, Self::I>,
        node: &Node<'_, Self::T, Self::E, Self::G, Self::I>,
        rel: Probability,
        smp: Probability,
    ) -> Utility {
        CfrFlow::recursed_value(self, root, node, rel, smp)
    }

    async fn dfs(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> (Policy<Self::E>, Utility) {
        CfrFlow::dfs(self, infoset)
    }

    async fn expected_value(&self, root: &Node<'_, Self::T, Self::E, Self::G, Self::I>) -> Utility {
        CfrFlow::expected_value(self, root)
    }

    async fn cfactual_value(&self, root: &Node<'_, Self::T, Self::E, Self::G, Self::I>, edge: &Self::E) -> Utility {
        CfrFlow::cfactual_value(self, root, edge)
    }

    async fn infoset_value(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Utility {
        CfrFlow::infoset_value(self, infoset)
    }

    async fn regret_vector(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Policy<Self::E> {
        CfrFlow::regret_vector(self, infoset)
    }

    async fn policy_vector(&self, infoset: &InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Policy<Self::E> {
        CfrFlow::policy_vector(self, infoset)
    }
}
