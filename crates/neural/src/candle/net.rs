use crate::*;
use mccfr::*;
use pokerkit::*;

/// Both Deep CFR networks as [`Architecture`]s on a [`Backend`], over an [`Embedder`]: one advantage net per player, refit
/// from scratch on its memory every time that player walks, and one strategy net fit at the end.
pub struct CfrNet<X, A = MLP, B = CPU>
where
    A: Architecture,
    B: Backend,
    X: Embedder,
{
    trainer: Trainer<X, B>,
    memories: Memories<X::F>,
    advantages: FxHashMap<TurnOf<X>, AdvantageNet<A>>,
    strategies: Option<StrategicNet<A>>,
}

impl<X, A, B> CfrSpec for CfrNet<X, A, B>
where
    X: Embedder,
    A: Architecture,
    B: Backend,
{
    type F = X::F;
}

impl<X, A, B> RegretNet for CfrNet<X, A, B>
where
    X: Embedder,
    A: Architecture,
    B: Backend,
{
    fn est_regret(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Regret> {
        match self.advantages.get(&self.trainer.embedder().turn(info)) {
            Some(net) => self.trainer.predict(net, info).expect("advantage forward pass"),
            None => info.menu().map(|e| (e, 0.)).collect(),
        }
    }

    fn see_regret(&mut self, walker: TurnOf<Self>, sample: Sample<Self::F, Regret>) {
        self.memories.see_regret(walker, sample);
    }

    fn fit_regret(&mut self, walker: TurnOf<Self>) {
        if let Some(rows) = self.memories.advantages(&walker) {
            let net = self
                .trainer
                .fit(rows, self.advantages.remove(&walker))
                .expect("advantage fit");
            self.advantages.insert(walker, net);
        }
    }
}

impl<X, A, B> StrategyNet for CfrNet<X, A, B>
where
    X: Embedder,
    A: Architecture,
    B: Backend,
{
    fn est_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        if let Some(net) = self.strategies.as_ref() {
            self.trainer.predict(net, info).expect("strategy forward pass")
        } else {
            let n = info.public().count() as Probability;
            info.menu().map(|e| (e, 1. / n)).collect()
        }
    }

    fn see_strategy(&mut self, sample: Sample<Self::F, Policy>) {
        self.memories.see_strategy(sample);
    }

    fn fit_strategy(&mut self) {
        self.strategies = Some(
            self.trainer
                .fit(self.memories.strategies(), self.strategies.take())
                .expect("strategy fit"),
        );
    }
}

impl<X, A, B> Default for CfrNet<X, A, B>
where
    X: Embedder + Default,
    A: Architecture,
    B: Backend + Default,
{
    fn default() -> Self {
        Self::new(X::default(), B::default())
    }
}

impl<X, A, B> CfrNet<X, A, B>
where
    X: Embedder,
    A: Architecture,
    B: Backend,
{
    pub fn new(embedder: X, backend: B) -> Self {
        Self {
            memories: Memories::new(DeepHyperParams::get().capacity()),
            trainer: Trainer::new(embedder, backend),
            advantages: FxHashMap::default(),
            strategies: None,
        }
    }

    /// Advantage nets refit from memory every epoch, so only the strategy net is saved.
    const STRATEGY: &'static str = "strategy.safetensors";

    /// Writes the fitted strategy net into `dir`; fails before the first `fit_strategy`.
    pub fn save(&self, dir: &std::path::Path) -> candle_core::Result<()> {
        std::fs::create_dir_all(dir)?;
        self.strategies
            .as_ref()
            .ok_or_else(|| candle_core::Error::Msg("no strategy net to save".into()))
            .and_then(|net| net.save(&dir.join(Self::STRATEGY)))
    }

    /// A net that plays the strategy saved in `dir`, with empty memories. The shape comes from
    /// `embedder` and [`DeepHyperParams`], which must match the run that saved it.
    pub fn load(embedder: X, backend: B, dir: &std::path::Path) -> candle_core::Result<Self> {
        let net = Self::new(embedder, backend);
        Ok(Self {
            strategies: Some(net.trainer.load(&dir.join(Self::STRATEGY))?),
            ..net
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuhn::*;
    use monge::Density;
    use roshambo::*;

    type KuhnNet = CfrNet<KuhnEmbedder>;

    impl KuhnNet {
        fn open() -> KuhnInfo {
            Composite::new(KuhnPublic::new(true, History::Open), Rank::J)
        }
    }

    #[test]
    fn untrained_plays_uniform() {
        let ref policy = KuhnNet::default().matched(&KuhnNet::open());
        assert_eq!(policy.density(&KuhnEdge::Check), 0.5);
        assert_eq!(policy.density(&KuhnEdge::Bet), 0.5);
    }

    #[test]
    fn regret_fit_reaches_the_weighted_mean() {
        let mut net = KuhnNet::default();
        let row = |t, check, bet| {
            Sample::new(KuhnNet::open(), t, Over::new(vec![(KuhnEdge::Check, check), (KuhnEdge::Bet, bet)]))
        };
        net.see_regret(KuhnTurn::Player(0), row(1, 3., 0.));
        net.see_regret(KuhnTurn::Player(0), row(3, -1., 4.));
        net.fit_regret(KuhnTurn::Player(0));
        let ref fitted = net.est_regret(&KuhnNet::open());
        assert!(fitted.density(&KuhnEdge::Check).abs() < 0.1, "{fitted:?}");
        assert!((fitted.density(&KuhnEdge::Bet) - 3.).abs() < 0.1, "{fitted:?}");
    }

    #[test]
    fn strategy_fit_reaches_the_weighted_mean() {
        let mut net = KuhnNet::default();
        let row = |t, check, bet| {
            Sample::new(KuhnNet::open(), t, Over::new(vec![(KuhnEdge::Check, check), (KuhnEdge::Bet, bet)]))
        };
        net.see_strategy(row(1, 1., 0.));
        net.see_strategy(row(3, 0., 1.));
        net.fit_strategy();
        let ref fitted = net.est_strategy(&KuhnNet::open());
        assert!((fitted.density(&KuhnEdge::Bet) - 0.75).abs() < 0.05, "{fitted:?}");
    }

    #[test]
    fn saved_strategy_loads_back() {
        let dir = std::env::temp_dir().join(format!("cfrnet-{}", std::process::id()));
        let mut net = KuhnNet::default();
        net.see_strategy(Sample::new(
            KuhnNet::open(),
            1,
            Over::new(vec![(KuhnEdge::Check, 0.3), (KuhnEdge::Bet, 0.7)]),
        ));
        net.fit_strategy();
        net.save(&dir).expect("save");
        let loaded = KuhnNet::load(KuhnEmbedder, CPU::default(), &dir).expect("load");
        std::fs::remove_dir_all(&dir).expect("clean up");
        let saved = net.est_strategy(&KuhnNet::open());
        let read = loaded.est_strategy(&KuhnNet::open());
        assert!(saved.iter().zip(read.iter()).all(|(a, b)| a == b), "{saved:?} vs {read:?}");
    }

    /// Held to the table-backed loop's bound, so it fails while the nets lag the tables: one
    /// release run landed on gap 0.0345 against the tables' 0.0019.
    #[test]
    #[ignore = "trains real networks: minutes in release; fails today, gap 0.0345 > 0.005"]
    fn rps_matches_nash() {
        let net = CfrNet::<RpsEmbedder>::default();
        let solver = DeepCfr::<RpsEncoder, _, ExternalSampling>::new(RpsEncoder, net).solve(1 << 12);
        let gap = RpsNash::gap(|turn, edge| solver.profile().averaged_policy(&turn, &edge));
        println!("candle deep cfr on rps: gap = {gap:.4}");
        assert!(gap < 0.005, "off the (0.4, 0.4, 0.2) equilibrium by {gap:.4}");
    }

    /// Held to the table-backed loop's bounds, so it fails while the nets lag the tables: one
    /// release run played Q|Open Check at 0.78 where equilibrium always checks.
    #[test]
    #[ignore = "trains real networks for ten minutes in release; fails today on Q|Open Check"]
    fn kuhn_matches_nash() {
        let net = KuhnNet::default();
        let solver = DeepCfr::<KuhnEncoder, _, ExternalSampling>::new(KuhnEncoder, net).solve(1 << 12);
        let e = solver.exploitability();
        println!("candle deep cfr on kuhn: exploitability = {e:.4}");
        let strays = KuhnNash::strays(|info, edge| solver.profile().averaged_policy(info, edge), 0.03);
        assert!(strays.is_empty(), "{strays:#?}");
        assert!(e < 0.015, "exploitability {e:.4} >= 0.015");
    }
}
