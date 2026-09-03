//! NLHE off-tree nesting: the `Augmentable` impl + the concrete `Nest` source.
//!
//! The wrapper *types* (`NestEdge`/`NestGame`/`NestInfo`/…) are game-agnostic
//! and live in [`subgame::nest`]. This file is the entire NLHE-specific surface:
//!
//! - [`Augmentable`] for [`NlheGame`] — the one seam, ~3 lines: apply a literal
//!   off-tree raise to the underlying `kicker::Game`.
//! - [`Nest`] — the concrete `SubGameSolver` source binding `&Nlhe` to those
//!   generic wrapper types (delegates encoding / payoffs / restrict to `Nlhe`).
use super::*;
use kicker::Action;
use kicker::Game;
use mccfr::*;
use subgame::*;

/// The one game-specific seam. The off-tree payload is a full [`Action`] — the
/// same thing `Game::translate` already hands back in its `Translated::Free`
/// arm — so `augment` is just a literal `Game::apply` with **no** knowledge of
/// *what* the action is (raise, or otherwise). Nothing here assumes "off-tree
/// == raise"; the caller decides the action, this only applies it.
impl Augmentable for NlheGame {
    type Off = Action;

    fn augment(&self, action: Action) -> Self {
        NlheGame::from(Game::from(*self).apply(action))
    }
}

/// Off-tree nesting source: `&Nlhe` viewed through the generic `subgame::nest`
/// wrapper types. `SubGameSolver` drives this exactly as it drives `Nlhe`.
pub struct Nest<'blueprint, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    nlhe: &'blueprint Nlhe<R, W, S>,
    encoder: NestEncoder<'blueprint, NlheEncoder>,
    blueprint: NestProfile<'blueprint, NlheProfile>,
    offtree: Action,
}

impl<'blueprint, R, W, S> Nest<'blueprint, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    pub fn new(nlhe: &'blueprint Nlhe<R, W, S>, offtree: Action) -> Self {
        Self {
            encoder: NestEncoder::new(&nlhe.encoder),
            blueprint: NestProfile::new(&nlhe.profile),
            nlhe,
            offtree,
        }
    }
}

impl<R, W, S> CfrEncoder for Nest<'_, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type T = NlheTurn;
    type E = NestEdge<NlheEdge, Action>;
    type G = NestGame<NlheGame>;
    type I = NestInfo<NlheInfo, Action>;

    fn seed(&self, game: &Self::G) -> Self::I {
        self.encoder.seed(game)
    }

    fn info(&self, tree: &Tree<Self::T, Self::E, Self::G, Self::I>, leaf: Leaf<Self::E, Self::G>) -> Self::I {
        self.encoder.info(tree, leaf)
    }

    fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        self.encoder.resume(past, game)
    }
}

impl<'blueprint, R, W, S> DepthSampler<{ pokerkit::FRONTIER_LEAVES }> for Nest<'blueprint, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type Blueprint = NestProfile<'blueprint, NlheProfile>;

    fn blueprint(&self) -> &Self::Blueprint {
        &self.blueprint
    }

    fn payoffs(
        &self,
        prefix: &Prefix<NlheTurn, NestEdge<NlheEdge, Action>>,
        game: &NestGame<NlheGame>,
        internal: NlheTurn,
    ) -> Payoffs<{ pokerkit::FRONTIER_LEAVES }> {
        // The construction prefix (root → entry) is canonical-only: the off-tree
        // edge is an in-tree menu option AT the entry, never in the prefix. So
        // projecting NestEdge → NlheEdge is a clean unwrap.
        let inner = prefix
            .as_slice()
            .iter()
            .map(|d| Descent(d.0, d.1.game().expect("nesting prefix is canonical-only")))
            .collect::<Vec<_>>();
        <Nlhe<R, W, S> as DepthSampler<{ pokerkit::FRONTIER_LEAVES }>>::payoffs(
            self.nlhe,
            &Prefix::new(inner),
            game.inner(),
            internal,
        )
    }
}

impl<R, W, S, const WORLDS: usize> WorldRestrict<WORLDS> for Nest<'_, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, WORLDS>,
        observed: &Self::G,
    ) -> Self::G {
        // Secret<Self> == Secret<Nlhe> == NlheSecret, so the belief passes
        // through. Wrap the restricted game as the entry node — (re-)setting the
        // is_entry flag once per sampled world (insight #1).
        NestGame::entry(
            <Nlhe<R, W, S> as WorldRestrict<WORLDS>>::restrict(self.nlhe, external, world, belief, observed.inner()),
            self.offtree,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time proof the nested solver satisfies `Solver + Harvest` —
    /// forcing every bound (incl. `CfrSolution<I = NestInfo<…>>` and `Sync`)
    /// without a hydrated blueprint. Runtime validation is N4 (post-retrain).
    #[allow(dead_code)]
    fn nested_solver_is_drivable() {
        fn is_solver<T>()
        where
            T: Solver,
        {
        }
        fn is_harvest<T>()
        where
            T: Harvest,
        {
        }
        type Nested = SubGameSolver<
            'static,
            { pokerkit::N_WORLDS },
            { pokerkit::FRONTIER_LEAVES },
            Nest<'static, mccfr::LinearRegret, mccfr::LinearWeight, mccfr::PluribusSampling>,
            NestInfo<NlheInfo, Action>,
            NlheSecret,
        >;
        is_solver::<Nested>();
        is_harvest::<Nested>();
    }
}
