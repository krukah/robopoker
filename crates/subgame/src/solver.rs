//! Subgame solver: safe subgame solving with optional depth-limited frontiers.
//!
//! `step()` samples a world from the belief distribution, restricts the observed game state
//! through [`WorldRestrict`], and runs standard CFR on the resulting tree. World separation
//! comes from [`WorldInfo`] tagging and frontier evaluation from [`DepthGame`] wrapping; with
//! `origin = None` frontier detection is off and this degenerates to pure safe subgame solving.
use std::collections::BTreeMap;

use crate::*;
use mccfr::*;
use pokerkit::Probability;

/// Solver for safe subgame solving with optional depth-limited frontiers.
///
/// Combines safety (worlds) and depth-limiting (frontiers) — use [`WorldSolver`]
/// for safety-only, or [`crate::depth::DepthSolver`] for depth-limiting only.
pub struct SubGameSolver<'blueprint, const W: usize, const L: usize, N>
where
    N: DepthSampler<L, Blueprint: Solution>,
{
    encoder: SubGameEncoder<'blueprint, N, L>,
    profile: WorldProfile<'blueprint, DepthView<'blueprint, N::Blueprint, L>>,
    belief: Belief<Secret<N>, W>,
    internal: TurnOf<N>,
    external: TurnOf<N>,
    recall: CfrRecall<GameOf<N>>,
    entry: DepthGame<GameOf<N>, L>,
    origin: Option<usize>,
}

impl<'blueprint, const W: usize, const L: usize, N> SubGameSolver<'blueprint, W, L, N>
where
    N: WorldRestrict<W> + DepthSampler<L, Blueprint: Solution>,
{
    pub fn new(
        source: &'blueprint N,
        external: TurnOf<N>,
        belief: Belief<Secret<N>, W>,
        recall: CfrRecall<GameOf<N>>,
    ) -> Self {
        Self::build(source, external, belief, recall, None)
    }

    pub fn with_origin(
        source: &'blueprint N,
        external: TurnOf<N>,
        belief: Belief<Secret<N>, W>,
        recall: CfrRecall<GameOf<N>>,
        origin: usize,
    ) -> Self {
        Self::build(source, external, belief, recall, Some(origin))
    }

    fn build(
        source: &'blueprint N,
        external: TurnOf<N>,
        belief: Belief<Secret<N>, W>,
        recall: CfrRecall<GameOf<N>>,
        origin: Option<usize>,
    ) -> Self {
        let world = Self::sample(&belief);
        let internal = (0..<TurnOf<N>>::players())
            .map(<TurnOf<N>>::from)
            .find(|t| t != &external)
            .expect("two player game");
        let inner = source.restrict(external, world, &belief, &recall.game());
        let entry = DepthGame::<_, L>::new(inner, internal, origin);
        let prefix = recall.descents().clone();
        let leaf_view: &'blueprint DepthView<'blueprint, N::Blueprint, L> =
            Box::leak(Box::new(DepthView::new(source.blueprint())));
        Self {
            encoder: SubGameEncoder::new(source, prefix, world),
            profile: WorldProfile::from(WorldView::new(leaf_view)),
            internal,
            external,
            belief,
            recall,
            entry,
            origin,
        }
    }

    pub fn into_profile(self) -> WorldProfile<'blueprint, DepthView<'blueprint, N::Blueprint, L>> {
        self.profile
    }

    fn encoder_mut(&mut self) -> &mut SubGameEncoder<'blueprint, N, L> {
        &mut self.encoder
    }

    fn belief(&self) -> &Belief<Secret<N>, W> {
        &self.belief
    }

    fn internal(&self) -> TurnOf<N> {
        self.internal
    }

    fn external(&self) -> TurnOf<N> {
        self.external
    }

    fn recall(&self) -> &CfrRecall<GameOf<N>> {
        &self.recall
    }

    fn origin(&self) -> Option<usize> {
        self.origin
    }

    fn entry_mut(&mut self) -> &mut DepthGame<GameOf<N>, L> {
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

impl<const W: usize, const L: usize, N> CfrSpec for SubGameSolver<'_, W, L, N>
where
    N: WorldRestrict<W> + DepthSampler<L, Blueprint: Solution> + Sync,
    N::Blueprint: Sync,
{
    type F = WorldInfo<DepthInfo<InfoOf<N>, L>>;
}

impl<'blueprint, const W: usize, const L: usize, N> Grower for SubGameSolver<'blueprint, W, L, N>
where
    N: WorldRestrict<W> + DepthSampler<L, Blueprint: Solution> + Sync,
    N::Blueprint: Sync,
{
    type P = WorldProfile<'blueprint, DepthView<'blueprint, N::Blueprint, L>>;
    type N = SubGameEncoder<'blueprint, N, L>;
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

impl<const W: usize, const L: usize, N> Solver for SubGameSolver<'_, W, L, N>
where
    N: WorldRestrict<W> + DepthSampler<L, Blueprint: Solution> + Sync,
    N::Blueprint: Sync,
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
        *self.entry_mut() = DepthGame::<_, L>::new(
            self.encoder()
                .inner()
                .restrict(self.external(), world, self.belief(), &self.recall().game()),
            self.internal(),
            self.origin(),
        );
        let updates = self.batch();
        let n = updates.len();
        for ref update in updates {
            self.update_regret(update);
            self.update_weight(update);
            self.update_payoff(update);
            self.update_visits(update);
        }
        tracing::trace!(
            "[full] t={:<6} world={:<2} infos={:<4} regret={:.4}",
            self.profile().t(),
            world.index(),
            n,
            self.profile().sum_regret(),
        );
        self.profile().metrics().inspect(|m: &&mccfr::Metrics| m.inc_epoch());
        self.advance();
    }
}

impl<const W: usize, const L: usize, N> Harvest for SubGameSolver<'_, W, L, N>
where
    N: WorldRestrict<W> + DepthSampler<L, Blueprint: Solution> + Sync,
    N::Blueprint: Sync,
{
    type Base = InfoOf<N>;

    fn harvest(&self, base: InfoOf<N>) -> Harvested<EdgeOf<N>> {
        let depth = base.game::<L>();
        let refined: BTreeMap<EdgeOf<N>, Probability> = (0..W)
            .map(World::from)
            .flat_map(|w| {
                self.profile()
                    .iterated_strategy(&depth.world(w))
                    .into_iter()
                    .filter_map(|(e, p)| e.game().map(|e| (e, p)))
            })
            .fold(BTreeMap::new(), |mut acc, (e, p)| {
                *acc.entry(e).or_insert(0.0) += p / W as Probability;
                acc
            });
        let visits = refined
            .keys()
            .map(|e| {
                let v = (0..W)
                    .map(World::from)
                    .map(|w| self.profile().cum_visits(&depth.world(w), &DepthEdge::Game(*e)))
                    .sum::<u32>();
                (*e, v)
            })
            .collect::<BTreeMap<EdgeOf<N>, u32>>();
        let regret = refined
            .keys()
            .flat_map(|e| {
                (0..W).map(World::from).map(move |w| {
                    self.profile()
                        .cum_regret(&depth.world(w), &DepthEdge::Game(*e))
                        .max(0.0)
                })
            })
            .sum();
        Harvested::new(refined, visits, regret)
    }
}
