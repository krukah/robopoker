//! Depth-limited solver (no world safety).
use std::collections::BTreeMap;

use crate::*;
use mccfr::*;
use pokerkit::Probability;

pub struct DepthSolver<'blueprint, N, const D: usize>
where
    N: DepthSampler<D>,
{
    encoder: DepthEncoder<'blueprint, N, D>,
    profile: DepthProfile<'blueprint, N::Blueprint, D>,
    entry: DepthGame<GameOf<N>, D>,
}

impl<'blueprint, N, const D: usize> DepthSolver<'blueprint, N, D>
where
    N: DepthSampler<D>,
{
    pub fn new(source: &'blueprint N, prefix: Prefix<GameOf<N>>, internal: TurnOf<N>, entry: GameOf<N>) -> Self {
        let origin = Some(entry.depth());
        Self {
            encoder: DepthEncoder::new(source, prefix),
            profile: DepthProfile::from(DepthView::new(source.blueprint())),
            entry: DepthGame::new(entry, internal, origin),
        }
    }

    pub fn into_profile(self) -> DepthProfile<'blueprint, N::Blueprint, D> {
        self.profile
    }
}

impl<N, const D: usize> CfrSpec for DepthSolver<'_, N, D>
where
    N: DepthSampler<D> + Sync,
    N::Blueprint: Clock + Sync,
{
    type F = DepthInfo<InfoOf<N>, D>;
}

impl<'blueprint, N, const D: usize> Grower for DepthSolver<'blueprint, N, D>
where
    N: DepthSampler<D> + Sync,
    N::Blueprint: Clock + Sync,
{
    type P = DepthProfile<'blueprint, N::Blueprint, D>;
    type N = DepthEncoder<'blueprint, N, D>;
    type S = ExternalSampling;

    fn encoder(&self) -> &Self::N {
        &self.encoder
    }

    fn profile(&self) -> &Self::P {
        &self.profile
    }

    fn root(&self) -> GameOf<Self> {
        self.entry
    }
}

impl<N, const D: usize> Solver for DepthSolver<'_, N, D>
where
    N: DepthSampler<D> + Sync,
    N::Blueprint: Clock + Sync,
{
    type R = SummedRegret;
    type W = LinearWeight;

    fn batch_size() -> usize {
        1
    }

    fn advance(&mut self) {
        self.storage().increment();
    }

    fn storage(&mut self) -> &mut Self::P {
        &mut self.profile
    }

    fn step(&mut self) {
        let updates = self.batch();
        let n = updates.len();
        for ref update in updates {
            self.update_regret(update);
            self.update_weight(update);
            self.update_payoff(update);
            self.update_visits(update);
        }
        tracing::trace!("[leaf] t={:<6} infos={:<4} regret={:.4}", self.profile().t(), n, self.profile().sum_regret(),);
        self.profile().metrics().into_iter().for_each(Metrics::inc_epoch);
        self.advance();
    }
}

impl<N, const D: usize> Harvest for DepthSolver<'_, N, D>
where
    N: DepthSampler<D> + Sync,
    N::Blueprint: Clock + Sync,
{
    type Base = InfoOf<N>;

    fn harvest(&self, base: InfoOf<N>) -> Harvested<EdgeOf<N>> {
        let info = base.game::<D>();
        let refined = self
            .profile()
            .iterated_strategy(&info)
            .into_iter()
            .filter_map(|(e, p)| e.game().map(|e| (e, p)))
            .collect::<BTreeMap<EdgeOf<N>, Probability>>();
        let visits = refined
            .keys()
            .map(|e| (*e, self.profile().cum_visits(&info, &DepthEdge::Game(*e))))
            .collect::<BTreeMap<EdgeOf<N>, u32>>();
        let regret = refined
            .keys()
            .map(|e| self.profile().cum_regret(&info, &DepthEdge::Game(*e)).max(0.0))
            .sum();
        Harvested::new(refined, visits, regret)
    }
}
