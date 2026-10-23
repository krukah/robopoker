use crate::*;
use monge::Density;
use pokerkit::*;

/// Async variant of [`Solution`] for database-backed or parallel training.
///
/// CFR is identical whether data lives in memory or a database — only access differs. So in-
/// memory profiles get this trait free via the blanket impl below, while DB-backed workers
/// implement it directly with real async I/O.
#[async_trait::async_trait]
pub trait AsyncProfile: CfrSpec + Send + Sync {
    /// Current traversing player.
    fn traverser(&self) -> TurnOf<Self>;
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
    async fn policy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy>;
    /// Exploration-adjusted sampling distribution.
    async fn sample(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy>;
    /// Historical average strategy (Nash approximation).
    async fn advice(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy>;
    /// Fused cfactual/sampling reach in one upward pass, batch-fetching
    /// policies and samples via `join_all` before folding both products.
    async fn ancestor_reach(&self, root: &Node<'_, Self::F>) -> Utility {
        let path = root
            .decisions()
            .filter(|choice| choice.turn() != self.traverser())
            .collect::<Vec<_>>();
        let (policies, samples) = futures::future::join(
            futures::future::join_all(path.iter().map(|choice| self.policy(choice.info()))),
            futures::future::join_all(path.iter().map(|choice| self.sample(choice.info()))),
        )
        .await;
        let (cf, sm) = path
            .iter()
            .zip(policies.iter())
            .zip(samples.iter())
            .fold((1.0, 1.0), |(cf, sm), ((choice, pol), smp)| {
                (cf * pol.density(choice.edge()), sm * smp.density(choice.edge()))
            });
        cf / sm
    }
    /// Recursive DFS accumulating reach during descent. Fetches policy (+ sample
    /// if non-walker) once per internal node, then `.density(edge)` per child.
    async fn recursed_value(
        &self,
        root: &Node<'_, Self::F>,
        node: &Node<'_, Self::F>,
        rel: Probability,
        smp: Probability,
    ) -> Utility {
        if node.width() == 0 {
            return rel / smp * node.game().payoff(root.game().turn());
        }
        let chance = node.game().turn() == <TurnOf<Self>>::chance();
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
    async fn dfs(&self, infoset: &InfoSet<Self::F>) -> (Over<EdgeOf<Self>, Regret>, Utility) {
        let span = infoset.span();
        let mut regrets = std::collections::HashMap::<EdgeOf<Self>, Utility>::new();
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
    async fn expected_value(&self, root: &Node<'_, Self::F>) -> Utility {
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
    async fn cfactual_value(&self, root: &Node<'_, Self::F>, edge: &EdgeOf<Self>) -> Utility {
        debug_assert_eq!(self.traverser(), root.game().turn());
        let reach = self.ancestor_reach(root).await;
        let child = root.step(edge).expect("edge belongs to outgoing branches");
        reach * self.recursed_value(root, &child, 1.0, 1.0).await
    }
    /// Expected value of an information set under current strategy.
    async fn infoset_value(&self, infoset: &InfoSet<Self::F>) -> Utility {
        self.dfs(infoset).await.1
    }
    /// Regret gains for all edges in an information set.
    async fn regret_vector(&self, infoset: &InfoSet<Self::F>) -> Over<EdgeOf<Self>, Regret> {
        self.dfs(infoset).await.0
    }
    async fn policy_vector(&self, infoset: &InfoSet<Self::F>) -> Over<EdgeOf<Self>, Policy> {
        self.policy(&infoset.info()).await
    }
}

/// Blanket [`AsyncProfile`] for all [`Solution`] types: every method delegates
/// to its synchronous counterpart, so in-memory profiles work in async contexts.
#[async_trait::async_trait]
impl<P> AsyncProfile for P
where
    P: Solution + Send + Sync,
{
    fn traverser(&self) -> TurnOf<Self> {
        Clock::walker(self)
    }

    fn iteration(&self) -> usize {
        Profile::t(self)
    }

    fn temperature(&self) -> Entropy {
        Clock::temperature(self)
    }

    fn smoothing(&self) -> Energy {
        Clock::smoothing(self)
    }

    fn curiosity(&self) -> Probability {
        Clock::curiosity(self)
    }

    async fn policy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        self.iterated_strategy(info)
    }

    async fn sample(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        self.sampling_strategy(info)
    }

    async fn advice(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        self.averaged_strategy(info)
    }

    async fn ancestor_reach(&self, root: &Node<'_, Self::F>) -> Utility {
        Flow::ancestor_reach(self, root)
    }

    async fn recursed_value(
        &self,
        root: &Node<'_, Self::F>,
        node: &Node<'_, Self::F>,
        rel: Probability,
        smp: Probability,
    ) -> Utility {
        Flow::recursed_value(self, root, node, rel, smp)
    }

    async fn dfs(&self, infoset: &InfoSet<Self::F>) -> (Over<EdgeOf<Self>, Regret>, Utility) {
        Flow::dfs(self, infoset)
    }

    async fn expected_value(&self, root: &Node<'_, Self::F>) -> Utility {
        Flow::expected_value(self, root)
    }

    async fn cfactual_value(&self, root: &Node<'_, Self::F>, edge: &EdgeOf<Self>) -> Utility {
        Flow::cfactual_value(self, root, edge)
    }

    async fn infoset_value(&self, infoset: &InfoSet<Self::F>) -> Utility {
        Flow::infoset_value(self, infoset)
    }

    async fn regret_vector(&self, infoset: &InfoSet<Self::F>) -> Over<EdgeOf<Self>, Regret> {
        Flow::regret_vector(self, infoset)
    }

    async fn policy_vector(&self, infoset: &InfoSet<Self::F>) -> Over<EdgeOf<Self>, Policy> {
        Flow::policy_vector(self, infoset)
    }
}
