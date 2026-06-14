//! Subgame solver: safe subgame solving with optional depth-limited frontiers.
//!
//! The solver overrides `step()` to:
//! 1. Sample a world from the belief distribution
//! 2. Restrict the observed game state via the [`WorldRestrict`] trait
//! 3. Run standard CFR on the resolve-phase tree
//!
//! World separation is achieved through [`WorldInfo`] tagging.
//! Depth-limited frontier evaluation is achieved through [`DepthGame`]
//! wrapping. When `origin = None`, frontier detection is disabled
//! and the solver degenerates to pure safe subgame solving.
use std::collections::BTreeMap;

use crate::SubGameEncoder;
use atlas::*;
use fulcrum::Probability;
use horizon::*;
use regret::*;

/// Solver for safe subgame solving with optional depth-limited frontiers.
///
/// Combines safety (worlds) and depth-limiting (frontiers) — use [`WorldSolver`]
/// for safety-only, or [`horizon::DepthSolver`] for depth-limiting only.
pub struct SubGameSolver<'blueprint, const W: usize, const L: usize, N, I, Y>
where
    N: DepthSampler<L, Blueprint: CfrSolution<I = I>>,
    N: WorldRestrict<W, I = I>,
    I: CfrInfo<E = N::E, T = N::T, Y = Y>,
    Y: CfrSecret,
{
    encoder: SubGameEncoder<'blueprint, N, L>,
    profile: WorldProfile<'blueprint, DepthView<'blueprint, N::Blueprint, L>>,
    belief: Belief<Y, W>,
    internal: N::T,
    external: N::T,
    recall: CfrRecall<N::G>,
    entry: DepthGame<N::G, L>,
    origin: Option<usize>,
}

impl<'blueprint, const W: usize, const L: usize, N, I, Y> SubGameSolver<'blueprint, W, L, N, I, Y>
where
    N: WorldRestrict<W, I = I> + DepthSampler<L, Blueprint: CfrSolution<I = I>>,
    I: CfrInfo<E = N::E, T = N::T, Y = Y>,
    Y: CfrSecret,
{
    pub fn new(source: &'blueprint N, external: N::T, belief: Belief<Y, W>, recall: CfrRecall<N::G>) -> Self {
        Self::build(source, external, belief, recall, None)
    }

    pub fn with_origin(
        source: &'blueprint N,
        external: N::T,
        belief: Belief<Y, W>,
        recall: CfrRecall<N::G>,
        origin: usize,
    ) -> Self {
        Self::build(source, external, belief, recall, Some(origin))
    }

    fn build(
        source: &'blueprint N,
        external: N::T,
        belief: Belief<Y, W>,
        recall: CfrRecall<N::G>,
        origin: Option<usize>,
    ) -> Self {
        let world = Self::sample(&belief);
        let internal = (0..N::T::players())
            .map(N::T::from)
            .find(|t| t != &external)
            .expect("two player game");
        let inner = source.restrict(external, world, &belief, &recall.game());
        let entry = DepthGame::<_, L>::new(inner, internal, origin);
        let prefix = recall.descents().to_vec();
        let leaf_view: &'blueprint DepthView<'blueprint, N::Blueprint, L> =
            Box::leak(Box::new(DepthView::new(source.blueprint())));
        Self {
            encoder: SubGameEncoder::new(source, prefix, world),
            profile: WorldProfile::new(leaf_view),
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

    fn sample(belief: &Belief<Y, W>) -> World {
        use rand::distr::weighted::WeightedIndex;
        use rand::prelude::*;
        WeightedIndex::new(belief.weights())
            .map(|d| World::from(d.sample(&mut rand::rng())))
            .expect("nonempty weights")
    }
}

impl<'blueprint, const W: usize, const L: usize, N, I, Y> Solver for SubGameSolver<'blueprint, W, L, N, I, Y>
where
    N: WorldRestrict<W, I = I> + DepthSampler<L, Blueprint: CfrSolution<I = I>> + Sync,
    N::Blueprint: Sync,
    I: CfrInfo<E = N::E, T = N::T, Y = Y>,
    Y: CfrSecret,
{
    type T = N::T;
    type E = DepthEdge<N::E, L>;
    type G = DepthGame<N::G, L>;
    type I = WorldInfo<DepthInfo<I, L>>;
    type X = DepthPublic<I::X, L>;
    type Y = I::Y;
    type P = WorldProfile<'blueprint, DepthView<'blueprint, N::Blueprint, L>>;
    type N = SubGameEncoder<'blueprint, N, L>;
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
        let inner = self
            .encoder
            .inner()
            .restrict(self.external, world, &self.belief, &self.recall.game());
        self.entry = DepthGame::<_, L>::new(inner, self.internal, self.origin);
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
            self.profile.t(),
            world.index(),
            n,
            self.profile.sum_regret(),
        );
        self.profile().metrics().inspect(|m: &&regret::Metrics| m.inc_epoch());
        self.advance();
    }
}

impl<const W: usize, const L: usize, N, I, Y> Harvest for SubGameSolver<'_, W, L, N, I, Y>
where
    N: WorldRestrict<W, I = I> + DepthSampler<L, Blueprint: CfrSolution<I = I>> + Sync,
    N::Blueprint: Sync,
    I: CfrInfo<E = N::E, T = N::T, Y = Y>,
    Y: CfrSecret,
{
    type Base = I;
    type Edge = N::E;

    fn harvest(&self, base: I) -> Harvested<N::E> {
        let depth = DepthInfo::<_, L>::Game(base);
        let refined: BTreeMap<N::E, Probability> = (0..W)
            .map(World::from)
            .flat_map(|w| {
                self.profile()
                    .iterated_distribution(&WorldInfo::new(w, depth))
                    .into_iter()
                    .filter_map(|(e, p)| match e {
                        DepthEdge::Game(e) => Some((e, p)),
                        _ => None,
                    })
            })
            .fold(BTreeMap::new(), |mut acc, (e, p)| {
                *acc.entry(e).or_insert(0.0) += p / W as Probability;
                acc
            });
        let visits: BTreeMap<N::E, u32> = refined
            .keys()
            .map(|e| {
                let v = (0..W)
                    .map(World::from)
                    .map(|w| {
                        self.profile()
                            .cum_visits(&WorldInfo::new(w, depth), &DepthEdge::Game(*e))
                    })
                    .sum::<u32>();
                (*e, v)
            })
            .collect();
        let regret = refined
            .keys()
            .flat_map(|e| {
                (0..W).map(World::from).map(move |w| {
                    self.profile()
                        .cum_regret(&WorldInfo::new(w, depth), &DepthEdge::Game(*e))
                        .max(0.0)
                })
            })
            .sum();
        Harvested {
            refined,
            visits,
            regret,
        }
    }
}
