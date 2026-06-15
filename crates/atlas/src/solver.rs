//! Safe subgame solver (no depth limiting).
//!
//! Implements the safe subgame solving technique using world sampling
//! and per-world regret separation, without any frontier/depth-limiting
//! machinery. The tree expands fully to terminal nodes.
//!
//! Use [`crate::SubGameSolver`] if you also want depth-limited frontier
//! evaluation, or [`horizon::DepthSolver`] for depth-limiting alone.
use std::collections::BTreeMap;

use crate::*;
use mccfr::*;
use pokerkit::Probability;

/// Safe subgame solver without depth limiting.
///
/// Samples a world from the belief partition, restricts the opponent's
/// hidden state to that world via [`WorldRestrict`], and runs CFR on the
/// full (non-depth-limited) subgame tree with per-world info sets.
pub struct WorldSolver<'blueprint, const W: usize, P, N, I, Y>
where
    P: CfrSolution<I = I>,
    N: WorldRestrict<W, T = P::T, E = P::E, G = P::G, I = I>,
    I: CfrInfo<E = P::E, T = P::T, Y = Y>,
    Y: CfrSecret,
{
    encoder: WorldEncoder<'blueprint, N>,
    profile: WorldProfile<'blueprint, P>,
    belief: Belief<Y, W>,
    external: P::T,
    recall: CfrRecall<P::G>,
    entry: P::G,
}

impl<'blueprint, const W: usize, P, N, I, Y> WorldSolver<'blueprint, W, P, N, I, Y>
where
    P: CfrSolution<I = I>,
    N: WorldRestrict<W, T = P::T, E = P::E, G = P::G, I = I>,
    I: CfrInfo<E = P::E, T = P::T, Y = Y>,
    Y: CfrSecret,
{
    pub fn new(
        encoder: &'blueprint N,
        profile: &'blueprint P,
        external: P::T,
        belief: Belief<Y, W>,
        recall: CfrRecall<P::G>,
    ) -> Self {
        let world = Self::sample(&belief);
        let entry = encoder.restrict(external, world, &belief, &recall.game());
        let prefix = recall.descents().to_vec();
        Self {
            encoder: WorldEncoder::new(encoder, prefix, world),
            profile: WorldProfile::new(profile),
            belief,
            external,
            recall,
            entry,
        }
    }

    pub fn into_profile(self) -> WorldProfile<'blueprint, P> {
        self.profile
    }

    fn sample(belief: &Belief<Y, W>) -> World {
        use rand::distr::weighted::WeightedIndex;
        use rand::prelude::*;
        WeightedIndex::new(belief.weights())
            .map(|d| World::from(d.sample(&mut rand::rng())))
            .expect("nonempty weights")
    }
}

impl<'blueprint, const W: usize, P, N, I, Y> Solver for WorldSolver<'blueprint, W, P, N, I, Y>
where
    P: CfrSolution<I = I> + Sync,
    N: WorldRestrict<W, T = P::T, E = P::E, G = P::G, I = I> + Sync,
    I: CfrInfo<E = P::E, T = P::T, Y = Y>,
    Y: CfrSecret,
{
    type T = P::T;
    type E = P::E;
    type G = P::G;
    type I = WorldInfo<I>;
    type X = I::X;
    type Y = I::Y;
    type P = WorldProfile<'blueprint, P>;
    type N = WorldEncoder<'blueprint, N>;
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
        let world = Self::sample(&self.belief);
        self.encoder.with_world(world);
        self.entry = self
            .encoder
            .inner()
            .restrict(self.external, world, &self.belief, &self.recall.game());
        let updates = self.batch();
        let n = updates.len();
        for ref update in updates {
            self.update_regret(update);
            self.update_weight(update);
            self.update_payoff(update);
            self.update_visits(update);
        }
        tracing::trace!(
            "[safe] t={:<6} world={:<2} infos={:<4} regret={:.4}",
            self.profile.t(),
            world.index(),
            n,
            self.profile.sum_regret(),
        );
        self.profile().metrics().inspect(|m| m.inc_epoch());
        self.advance();
    }
}

impl<const W: usize, P, N, I, Y> Harvest for WorldSolver<'_, W, P, N, I, Y>
where
    P: CfrSolution<I = I> + Sync,
    N: WorldRestrict<W, T = P::T, E = P::E, G = P::G, I = I> + Sync,
    I: CfrInfo<E = P::E, T = P::T, Y = Y>,
    Y: CfrSecret,
{
    type Base = I;
    type Edge = P::E;

    fn harvest(&self, base: I) -> Harvested<P::E> {
        let refined: BTreeMap<P::E, Probability> = (0..W)
            .map(World::from)
            .flat_map(|w| {
                self.profile()
                    .iterated_distribution(&WorldInfo::new(w, base))
                    .into_iter()
            })
            .fold(BTreeMap::new(), |mut acc, (e, p)| {
                *acc.entry(e).or_insert(0.0) += p / W as Probability;
                acc
            });
        let visits: BTreeMap<P::E, u32> = refined
            .keys()
            .map(|e| {
                let v = (0..W)
                    .map(World::from)
                    .map(|w| self.profile().cum_visits(&WorldInfo::new(w, base), e))
                    .sum::<u32>();
                (*e, v)
            })
            .collect();
        let regret = refined
            .keys()
            .flat_map(|e| {
                (0..W)
                    .map(World::from)
                    .map(move |w| self.profile().cum_regret(&WorldInfo::new(w, base), e).max(0.0))
            })
            .sum();
        Harvested {
            refined,
            visits,
            regret,
        }
    }
}
