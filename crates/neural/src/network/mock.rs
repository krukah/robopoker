use crate::*;
use mccfr::*;
use pokerkit::*;

/// Both Deep CFR networks as tables. Each fit is the exact minimizer of the paper's `t`-weighted
/// MSE over its memory — the `t`-weighted mean target per infoset — so with unbounded memory it
/// reproduces sampled Linear CFR, the limit a perfectly expressive network approaches.
pub struct MockNet<F>
where
    F: CfrInfo,
{
    memories: Memories<F>,
    advantages: FxHashMap<TurnOf<F>, FxHashMap<F, Over<EdgeOf<F>, Regret>>>,
    strategies: FxHashMap<F, Over<EdgeOf<F>, Policy>>,
}

impl<F> CfrSpec for MockNet<F>
where
    F: CfrInfo,
{
    type F = F;
}

impl<F> RegretNet for MockNet<F>
where
    F: CfrInfo,
{
    fn est_regret(&self, info: &F) -> Over<EdgeOf<F>, Regret> {
        self.advantages()
            .values()
            .find_map(|table| table.get(info))
            .cloned()
            .unwrap_or_else(|| info.menu().map(|e| (e, 0.)).collect())
    }

    fn see_regret(&mut self, walker: TurnOf<F>, sample: Sample<F, Regret>) {
        self.memories_mut().see_regret(walker, sample);
    }

    fn fit_regret(&mut self, walker: TurnOf<F>) {
        let table = self
            .memories()
            .advantages(&walker)
            .map(Reservoir::regress)
            .unwrap_or_default();
        self.advantages_mut().insert(walker, table);
    }
}

impl<F> StrategyNet for MockNet<F>
where
    F: CfrInfo,
{
    fn est_strategy(&self, info: &F) -> Over<EdgeOf<F>, Policy> {
        self.strategies().get(info).cloned().unwrap_or_else(|| {
            let n = info.public().count() as Probability;
            info.menu().map(|e| (e, 1. / n)).collect()
        })
    }

    fn see_strategy(&mut self, sample: Sample<F, Policy>) {
        self.memories_mut().see_strategy(sample);
    }

    fn fit_strategy(&mut self) {
        *self.strategies_mut() = self.memories().strategies().regress();
    }
}

impl<F> Default for MockNet<F>
where
    F: CfrInfo,
{
    fn default() -> Self {
        Self::new(DeepHyperParams::get().capacity())
    }
}

impl<F> MockNet<F>
where
    F: CfrInfo,
{
    /// `capacity` bounds each memory: one `M_V,p` per player, plus `M_Π`.
    pub fn new(capacity: usize) -> Self {
        Self {
            memories: Memories::new(capacity),
            advantages: FxHashMap::default(),
            strategies: FxHashMap::default(),
        }
    }

    fn memories(&self) -> &Memories<F> {
        &self.memories
    }

    fn memories_mut(&mut self) -> &mut Memories<F> {
        &mut self.memories
    }

    fn advantages(&self) -> &FxHashMap<TurnOf<F>, FxHashMap<F, Over<EdgeOf<F>, Regret>>> {
        &self.advantages
    }

    fn advantages_mut(&mut self) -> &mut FxHashMap<TurnOf<F>, FxHashMap<F, Over<EdgeOf<F>, Regret>>> {
        &mut self.advantages
    }

    fn strategies(&self) -> &FxHashMap<F, Over<EdgeOf<F>, Policy>> {
        &self.strategies
    }

    fn strategies_mut(&mut self) -> &mut FxHashMap<F, Over<EdgeOf<F>, Policy>> {
        &mut self.strategies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuhn::*;
    use monge::Density;

    type KuhnNet = MockNet<KuhnInfo>;

    impl KuhnNet {
        fn open() -> KuhnInfo {
            Composite::new(KuhnPublic::new(true, History::Open), Rank::J)
        }

        fn sample(iteration: usize, check: Utility, bet: Utility) -> Sample<KuhnInfo, Regret> {
            Sample::new(Self::open(), iteration, Over::new(vec![(KuhnEdge::Check, check), (KuhnEdge::Bet, bet)]))
        }
    }

    #[test]
    fn untrained_plays_uniform() {
        let ref policy = KuhnNet::new(1 << 4).matched(&KuhnNet::open());
        assert_eq!(policy.density(&KuhnEdge::Check), 0.5);
        assert_eq!(policy.density(&KuhnEdge::Bet), 0.5);
    }

    #[test]
    fn fit_is_linearly_weighted_mean() {
        let mut net = KuhnNet::new(1 << 4);
        net.see_regret(KuhnTurn::Player(0), KuhnNet::sample(1, 3., 0.));
        net.see_regret(KuhnTurn::Player(0), KuhnNet::sample(3, -1., 4.));
        net.fit_regret(KuhnTurn::Player(0));
        let ref fitted = net.est_regret(&KuhnNet::open());
        assert_eq!(fitted.density(&KuhnEdge::Check), 0.);
        assert_eq!(fitted.density(&KuhnEdge::Bet), 3.);
    }

    #[test]
    fn nonpositive_advantages_play_argmax() {
        let mut net = KuhnNet::new(1 << 4);
        net.see_regret(KuhnTurn::Player(0), KuhnNet::sample(1, -2., -1.));
        net.fit_regret(KuhnTurn::Player(0));
        let ref policy = net.matched(&KuhnNet::open());
        assert_eq!(policy.density(&KuhnEdge::Check), 0.);
        assert_eq!(policy.density(&KuhnEdge::Bet), 1.);
    }
}
