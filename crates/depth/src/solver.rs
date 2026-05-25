//! Depth-limited solver (no world safety).
use std::collections::BTreeMap;

use crate::*;
use rbp_core::Probability;
use rbp_mccfr::*;

pub struct DepthSolver<'blueprint, N, const D: usize>
where
    N: DepthSampler<D>,
{
    encoder: DepthEncoder<'blueprint, N, D>,
    profile: DepthProfile<'blueprint, N::Blueprint, D>,
    entry: DepthGame<N::G, D>,
}

impl<'blueprint, N, const D: usize> DepthSolver<'blueprint, N, D>
where
    N: DepthSampler<D>,
{
    pub fn new(source: &'blueprint N, prefix: Vec<Descent<N::T, N::E>>, internal: N::T, entry: N::G) -> Self {
        let origin = Some(entry.depth());
        Self {
            encoder: DepthEncoder::new(source, prefix),
            profile: DepthProfile::new(source.blueprint()),
            entry: DepthGame::new(entry, internal, origin),
        }
    }

    pub fn into_profile(self) -> DepthProfile<'blueprint, N::Blueprint, D> {
        self.profile
    }
}

impl<'blueprint, N, const D: usize> Solver for DepthSolver<'blueprint, N, D>
where
    N: DepthSampler<D> + Sync,
    N::Blueprint: CfrSampling + Sync,
{
    type T = N::T;
    type E = DepthEdge<N::E, D>;
    type G = DepthGame<N::G, D>;
    type I = DepthInfo<N::I, D>;
    type X = DepthPublic<<N::I as CfrInfo>::X, D>;
    type Y = <N::I as CfrInfo>::Y;
    type P = DepthProfile<'blueprint, N::Blueprint, D>;
    type N = DepthEncoder<'blueprint, N, D>;
    type S = ExternalSampling;
    type R = SummedRegret;
    type W = LinearWeight;

    fn batch_size() -> usize {
        1
    }

    fn advance(&mut self) {
        self.profile.increment();
    }

    fn encoder(&self) -> &Self::N {
        &self.encoder
    }

    fn profile(&self) -> &Self::P {
        &self.profile
    }

    fn storage(&mut self) -> &mut Self::P {
        &mut self.profile
    }

    fn root(&self) -> Self::G {
        self.entry
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
        tracing::trace!("[leaf] t={:<6} infos={:<4} regret={:.4}", self.profile.t(), n, self.profile.sum_regret(),);
        self.profile().metrics().inspect(|m| m.inc_epoch());
        self.advance();
    }
}

impl<'blueprint, N, const D: usize> Harvest for DepthSolver<'blueprint, N, D>
where
    N: DepthSampler<D> + Sync,
    N::Blueprint: CfrSampling + Sync,
{
    type Base = N::I;
    type Edge = N::E;

    fn harvest(&self, base: N::I) -> Harvested<N::E> {
        let info = DepthInfo::<_, D>::Game(base);
        let refined: BTreeMap<N::E, Probability> = self
            .profile()
            .iterated_distribution(&info)
            .into_iter()
            .filter_map(|(e, p)| match e {
                DepthEdge::Game(e) => Some((e, p)),
                _ => None,
            })
            .collect();
        let visits: BTreeMap<N::E, u32> = refined
            .keys()
            .map(|e| (*e, self.profile().cum_visits(&info, &DepthEdge::Game(*e))))
            .collect();
        let regret = refined
            .keys()
            .map(|e| self.profile().cum_regret(&info, &DepthEdge::Game(*e)).max(0.0))
            .sum();
        Harvested {
            refined,
            visits,
            regret,
        }
    }
}
