use crate::*;
use mccfr::*;
use monge::Density;
use pokerkit::*;

/// Lends [`Profile`] and [`Clock`] to a pair of networks, so [`Flow`], [`Equilibrium`], and
/// every [`SamplingScheme`] run over function approximators exactly as they run over tables.
///
/// `Flow` asks one edge at a time; the memos hold each infoset's prediction until the net changes.
pub struct DeepProfile<N>
where
    N: RegretNet + StrategyNet,
{
    net: N,
    epoch: usize,
    current: Predictions<N>,
    average: Predictions<N>,
}

impl<N> CfrSpec for DeepProfile<N>
where
    N: RegretNet + StrategyNet,
{
    type F = N::F;
}

impl<N> Profile for DeepProfile<N>
where
    N: RegretNet + StrategyNet,
{
    fn t(&self) -> usize {
        self.epoch()
    }

    /// A network enumerates no infosets, so the total is undefined.
    fn sum_regret(&self) -> Utility {
        Utility::NAN
    }

    /// The current strategy `σᵗ`, which [`Flow::sampling_weight`] draws opponent actions from.
    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.current()
            .with(info, || self.net().matched(info), |p| p.density(edge))
    }

    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.net().est_regret(info).density(edge)
    }

    /// Deep CFR trains no value network, so frontier evaluation has nothing to read.
    fn cum_payoff(&self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> Utility {
        Utility::default()
    }

    fn cum_visits(&self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> u32 {
        0
    }

    /// The regret-matching numerator, normalized, so [`Flow::regret_denom`] is one.
    fn regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.current()
            .with(info, || self.net().matched(info), |p| p.density(edge))
    }

    fn iterated_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        self.current().with(info, || self.net().matched(info), Over::clone)
    }

    fn averaged_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        self.average().with(info, || self.net().est_strategy(info), Over::clone)
    }
}

/// Deep CFR's external sampling draws opponents on-policy from `σᵗ`, so exploration is neutral.
impl<N> Clock for DeepProfile<N>
where
    N: RegretNet + StrategyNet,
{
    fn increment(&mut self) {
        *self.epoch_mut() += 1;
    }

    fn temperature(&self) -> Entropy {
        1.0
    }

    fn smoothing(&self) -> Energy {
        0.0
    }

    fn curiosity(&self) -> Probability {
        0.0
    }
}

impl<N> Default for DeepProfile<N>
where
    N: RegretNet + StrategyNet + Default,
{
    fn default() -> Self {
        Self::new(N::default())
    }
}

impl<N> DeepProfile<N>
where
    N: RegretNet + StrategyNet,
{
    pub fn new(net: N) -> Self {
        Self {
            net,
            epoch: 0,
            current: Memo::default(),
            average: Memo::default(),
        }
    }

    pub fn net(&self) -> &N {
        &self.net
    }

    /// The only way to change the net, so it invalidates every cached prediction.
    pub fn net_mut(&mut self) -> &mut N {
        self.current_mut().clear();
        self.average_mut().clear();
        self.net_raw()
    }

    pub fn epoch(&self) -> usize {
        self.epoch
    }

    /// Feeds one epoch's rows to the nets: regrets into the walker's memory, strategies into `M_Π`.
    pub fn observe(&mut self, batch: TrainingBatch<N::F>) {
        batch.feed(self.walker(), self.net_mut());
    }

    /// Deep CFR's 1-based `t`: every player traverses once per iteration.
    pub fn iteration(&self) -> usize {
        self.epoch() / <TurnOf<N>>::players() + 1
    }

    fn epoch_mut(&mut self) -> &mut usize {
        &mut self.epoch
    }

    fn net_raw(&mut self) -> &mut N {
        &mut self.net
    }

    fn current(&self) -> &Predictions<N> {
        &self.current
    }

    fn current_mut(&mut self) -> &mut Predictions<N> {
        &mut self.current
    }

    fn average(&self) -> &Predictions<N> {
        &self.average
    }

    fn average_mut(&mut self) -> &mut Predictions<N> {
        &mut self.average
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuhn::*;

    type KuhnNet = Counted<MockNet<KuhnInfo>>;

    impl DeepProfile<KuhnNet> {
        fn kuhn() -> Self {
            Self::new(Counted::new(MockNet::new(1 << 4)))
        }

        fn open() -> KuhnInfo {
            Composite::new(KuhnPublic::new(true, History::Open), Rank::J)
        }
    }

    #[test]
    fn one_regret_pass_per_infoset() {
        let ref info = DeepProfile::<KuhnNet>::open();
        let profile = DeepProfile::<KuhnNet>::kuhn();
        info.menu().for_each(|ref e| {
            profile.regret(info, e);
            profile.cum_weight(info, e);
            profile.instant_policy(info, e);
            profile.sampling(info, e);
        });
        profile.iterated_strategy(info);
        assert_eq!(profile.net().regrets(), 1);
    }

    #[test]
    fn one_strategy_pass_per_infoset() {
        let ref info = DeepProfile::<KuhnNet>::open();
        let profile = DeepProfile::<KuhnNet>::kuhn();
        info.menu().for_each(|ref e| {
            profile.averaged_policy(info, e);
        });
        assert_eq!(profile.net().weights(), 1);
    }

    #[test]
    fn net_mut_invalidates() {
        let ref info = DeepProfile::<KuhnNet>::open();
        let mut profile = DeepProfile::<KuhnNet>::kuhn();
        assert_eq!(profile.iterated_strategy(info).density(&KuhnEdge::Bet), 0.5);
        let bet = Over::new(vec![(KuhnEdge::Check, -1.), (KuhnEdge::Bet, 1.)]);
        profile
            .net_mut()
            .see_regret(KuhnTurn::Player(0), Sample::new(*info, 1, bet));
        profile.net_mut().fit_regret(KuhnTurn::Player(0));
        assert_eq!(profile.iterated_strategy(info).density(&KuhnEdge::Bet), 1.);
        assert_eq!(profile.net().regrets(), 2);
    }
}
