//! Safe subgame solver (no depth limiting).
//!
//! See [`crate::SubGameSolver`] for safety plus depth-limited frontiers, or
//! [`subgame::DepthSolver`] for depth-limiting alone.
use std::collections::BTreeMap;

use crate::*;
use mccfr::*;
use pokerkit::Probability;

/// Samples a world from the belief partition, restricts the opponent's hidden
/// state to it via [`WorldRestrict`], and runs CFR with per-world info sets on
/// the full tree, expanded all the way to terminal nodes.
pub struct WorldSolver<'blueprint, const W: usize, P, N>
where
    P: Solution,
    N: WorldRestrict<W, F = P::F>,
{
    encoder: WorldEncoder<'blueprint, N>,
    profile: WorldProfile<'blueprint, P>,
    belief: Belief<Secret<N>, W>,
    external: TurnOf<P>,
    recall: CfrRecall<GameOf<P>>,
    entry: GameOf<P>,
}

impl<'blueprint, const W: usize, P, N> WorldSolver<'blueprint, W, P, N>
where
    P: Solution,
    N: WorldRestrict<W, F = P::F>,
{
    pub fn new(
        encoder: &'blueprint N,
        profile: &'blueprint P,
        external: TurnOf<P>,
        belief: Belief<Secret<N>, W>,
        recall: CfrRecall<GameOf<P>>,
    ) -> Self {
        let world = Self::sample(&belief);
        let entry = encoder.restrict(external, world, &belief, &recall.game());
        let prefix = recall.descents().clone();
        Self {
            encoder: WorldEncoder::new(encoder, prefix, world),
            profile: WorldProfile::from(WorldView::new(profile)),
            belief,
            external,
            recall,
            entry,
        }
    }

    pub fn into_profile(self) -> WorldProfile<'blueprint, P> {
        self.profile
    }

    fn encoder_mut(&mut self) -> &mut WorldEncoder<'blueprint, N> {
        &mut self.encoder
    }

    fn belief(&self) -> &Belief<Secret<N>, W> {
        &self.belief
    }

    fn external(&self) -> TurnOf<P> {
        self.external
    }

    fn recall(&self) -> &CfrRecall<GameOf<P>> {
        &self.recall
    }

    fn entry_mut(&mut self) -> &mut GameOf<P> {
        &mut self.entry
    }

    fn sample(belief: &Belief<Secret<N>, W>) -> World {
        use rand::distr::weighted::WeightedIndex;
        use rand::prelude::*;
        WeightedIndex::new(belief.weights())
            .map(|d| World::from(d.sample(&mut rand::rng())))
            .expect("nonempty weights")
    }
}

impl<const W: usize, P, N> CfrSpec for WorldSolver<'_, W, P, N>
where
    P: Solution + Sync,
    N: WorldRestrict<W, F = P::F> + Sync,
{
    type F = WorldInfo<InfoOf<P>>;
}

impl<'blueprint, const W: usize, P, N> Grower for WorldSolver<'blueprint, W, P, N>
where
    P: Solution + Sync,
    N: WorldRestrict<W, F = P::F> + Sync,
{
    type P = WorldProfile<'blueprint, P>;
    type N = WorldEncoder<'blueprint, N>;
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

impl<const W: usize, P, N> Solver for WorldSolver<'_, W, P, N>
where
    P: Solution + Sync,
    N: WorldRestrict<W, F = P::F> + Sync,
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
        let world = Self::sample(self.belief());
        self.encoder_mut().with_world(world);
        *self.entry_mut() =
            self.encoder()
                .inner()
                .restrict(self.external(), world, self.belief(), &self.recall().game());
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
            self.profile().t(),
            world.index(),
            n,
            self.profile().sum_regret(),
        );
        self.profile().metrics().into_iter().for_each(Metrics::inc_epoch);
        self.advance();
    }
}

impl<const W: usize, P, N> Harvest for WorldSolver<'_, W, P, N>
where
    P: Solution + Sync,
    N: WorldRestrict<W, F = P::F> + Sync,
{
    type Base = InfoOf<P>;

    fn harvest(&self, base: InfoOf<P>) -> Harvested<EdgeOf<P>> {
        let refined: BTreeMap<EdgeOf<P>, Probability> = (0..W)
            .map(World::from)
            .flat_map(|w| self.profile().iterated_strategy(&base.world(w)).into_iter())
            .fold(BTreeMap::new(), |mut acc, (e, p)| {
                *acc.entry(e).or_insert(0.0) += p / W as Probability;
                acc
            });
        let visits = refined
            .keys()
            .map(|e| {
                let v = (0..W)
                    .map(World::from)
                    .map(|w| self.profile().cum_visits(&base.world(w), e))
                    .sum::<u32>();
                (*e, v)
            })
            .collect::<BTreeMap<EdgeOf<P>, u32>>();
        let regret = refined
            .keys()
            .flat_map(|e| {
                (0..W)
                    .map(World::from)
                    .map(move |w| self.profile().cum_regret(&base.world(w), e).max(0.0))
            })
            .sum();
        Harvested::new(refined, visits, regret)
    }
}
