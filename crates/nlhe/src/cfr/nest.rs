//! NLHE off-tree nesting: the `Augmentable` impl + the concrete `Nest` source.
//!
//! The wrapper *types* (`NestEdge`/`NestGame`/`NestInfo`/…) are game-agnostic and live in
//! [`subgame::nest`].
//!
//! off-tree raise to the underlying `kicker::Game`.
//!
//! generic wrapper types (delegates encoding / payoffs / restrict to `Nlhe`).
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
    nlhe: &'blueprint Tabular<NlheEncoder, R, W, S, 128>,
    encoder: NestEncoder<'blueprint, NlheEncoder>,
    blueprint: NestProfile<'blueprint, Book<NlheInfo>>,
    offtree: Action,
}

impl<'blueprint, R, W, S> Nest<'blueprint, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    pub fn new(nlhe: &'blueprint Tabular<NlheEncoder, R, W, S, 128>, offtree: Action) -> Self {
        Self {
            encoder: NestEncoder::new(nlhe.encoder()),
            blueprint: NestProfile::from(NestView::new(nlhe.profile())),
            nlhe,
            offtree,
        }
    }

    fn nlhe(&self) -> &'blueprint Tabular<NlheEncoder, R, W, S, 128> {
        self.nlhe
    }

    fn encoder(&self) -> &NestEncoder<'blueprint, NlheEncoder> {
        &self.encoder
    }

    fn offtree(&self) -> Action {
        self.offtree
    }
}

impl<R, W, S> CfrSpec for Nest<'_, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type F = NestInfo<NlheInfo, Action>;
}

impl<R, W, S> Encoding for Nest<'_, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        self.encoder().public(game, past)
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        self.encoder().secret(game)
    }
}

impl<'blueprint, R, W, S> DepthSampler<{ pokerkit::FRONTIER_LEAVES }> for Nest<'blueprint, R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type Blueprint = NestProfile<'blueprint, Book<NlheInfo>>;

    fn blueprint(&self) -> &Self::Blueprint {
        &self.blueprint
    }

    fn payoffs(
        &self,
        prefix: &Prefix<NestGame<NlheGame>>,
        game: &NestGame<NlheGame>,
        internal: NlheTurn,
    ) -> Payoffs<{ pokerkit::FRONTIER_LEAVES }> {
        // The construction prefix (root → entry) is canonical-only: the off-tree edge is an in-
        // tree menu option AT the entry, never in the prefix.
        let inner = prefix
            .as_slice()
            .iter()
            .map(|d| Descent(d.0, d.1.game().expect("nesting prefix is canonical-only")))
            .collect::<Vec<_>>();
        <Tabular<NlheEncoder, R, W, S, 128> as DepthSampler<{ pokerkit::FRONTIER_LEAVES }>>::payoffs(
            self.nlhe(),
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
        external: TurnOf<Self>,
        world: World,
        belief: &Belief<Secret<Self>, WORLDS>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self> {
        // Secret<Self> == Secret<Nlhe> == NlheSecret, so the belief passes through.
        NestGame::entry(
            <Tabular<NlheEncoder, R, W, S, 128> as WorldRestrict<WORLDS>>::restrict(
                self.nlhe(),
                external,
                world,
                belief,
                observed.inner(),
            ),
            self.offtree(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Nest<'static, mccfr::LinearRegret, mccfr::LinearWeight, mccfr::PluribusSampling> {
        fn drivable<T>()
        where
            T: Solver + Harvest,
        {
        }
    }

    /// Compile-time proof the nested solver satisfies `Solver + Harvest`.
    #[test]
    fn nested_solver_is_drivable() {
        Nest::drivable::<
            SubGameSolver<
                'static,
                { pokerkit::N_WORLDS },
                { pokerkit::FRONTIER_LEAVES },
                Nest<'static, mccfr::LinearRegret, mccfr::LinearWeight, mccfr::PluribusSampling>,
            >,
        >();
    }
}
