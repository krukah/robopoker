//! [`Adapt`] — a subgame sampler generalized over its blueprint store.
//!
//! [`Nlhe`] hard-codes `Blueprint = NlheProfile`, the whole strategy table in
//! RAM. [`Adapt`] offers the same `adapt_leaf` / `adapt_safe` / `adapt_full`
//! surface over any `B` readable through [`RefProf`] / [`CfrSolution`] —
//! [`LazyBlueprint`] for DB-backed RAM-free play (aliased [`Lazy`]), or
//! [`NlheProfile`] moved in.
//!
//! Nothing in the existing solver zoo is touched: the subgame solvers are
//! already generic over sampler and blueprint, so they accept [`Adapt`]
//! verbatim. That is the payoff of the blueprint seam being `RefProf` rather
//! than an async parallel of the whole hierarchy.
use super::*;
use kicker::*;
use mccfr::*;
use pokerkit::*;
use subgame::*;

/// A subgame sampler bundling a borrowed encoder with an owned blueprint `B`.
///
/// Encoding delegates to the [`NlheEncoder`]; strategy lookups go through `B`.
/// Instantiate `B` with [`LazyBlueprint`] for RAM-free play or [`NlheProfile`]
/// for the in-memory table.
pub struct Adapt<'e, B> {
    encoder: &'e NlheEncoder,
    blueprint: B,
}

/// The DB-backed instantiation: a subgame sampler that never loads the
/// blueprint into RAM.
pub type Lazy<'e> = Adapt<'e, LazyBlueprint>;

impl<'e, B> Adapt<'e, B> {
    pub fn new(encoder: &'e NlheEncoder, blueprint: B) -> Self {
        Self { encoder, blueprint }
    }
}

impl Adapt<'_, LazyBlueprint> {
    /// Distinct blueprint infos pulled from the store this session — the
    /// realized working-set size, versus the whole table under [`Nlhe`].
    pub fn fetches(&self) -> usize {
        self.blueprint.fetches()
    }
}

impl<B> CfrEncoder for Adapt<'_, B> {
    type T = NlheTurn;
    type E = NlheEdge;
    type G = NlheGame;
    type I = NlheInfo;

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

impl<B> DepthSampler<{ pokerkit::FRONTIER_LEAVES }> for Adapt<'_, B>
where
    B: RefProf<T = NlheTurn, E = NlheEdge, G = NlheGame, I = NlheInfo>,
{
    type Blueprint = B;

    fn blueprint(&self) -> &Self::Blueprint {
        &self.blueprint
    }

    fn payoffs(
        &self,
        prefix: &Prefix<NlheTurn, NlheEdge>,
        game: &NlheGame,
        internal: NlheTurn,
    ) -> Payoffs<{ pokerkit::FRONTIER_LEAVES }> {
        let encoder = self.encoder;
        let ref profile = self.blueprint;
        let rollouts = FrontierHyperParams::get().rollouts();
        Payoffs::tabulate(|k, j| {
            (0..rollouts)
                .map(|_| encoder.biased_rollout(prefix, game, internal, k, j, profile))
                .sum::<Utility>()
                / rollouts as Utility
        })
    }
}

impl<B, const W: usize> WorldRestrict<W> for Adapt<'_, B> {
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &Self::G,
    ) -> Self::G {
        self.encoder.restrict(external, world, belief, observed)
    }
}

impl<B> Adapt<'_, B>
where
    B: RefProf<T = NlheTurn, E = NlheEdge, G = NlheGame, I = NlheInfo> + CfrSampling + Sync,
{
    /// Depth-limited solve rooted at `recall.head()`, reading strategy from
    /// `B`. Parallels [`Nlhe::adapt_leaf`].
    pub fn adapt_leaf(&self, recall: &Witness) -> DepthSolver<'_, Self, { pokerkit::FRONTIER_LEAVES }> {
        let internal = NlheTurn::from(recall.turn());
        let entry = NlheGame::from(recall.head());
        let prefix = subgame_descents(recall);
        DepthSolver::new(self, prefix, internal, entry)
    }

    /// Opponent posterior over abstraction buckets, from blueprint reach along
    /// the observed line. Parallels [`Nlhe::opponent_range`].
    fn opponent_range(&self, recall: &Witness) -> Posterior<NlheSecret> {
        let external = opposing(recall.turn());
        recall
            .possibilities()
            .into_iter()
            .map(|(obs, case)| (obs, self.reach(case, external)))
            .map(|(obs, reach)| (NlheSecret::from(self.encoder.abstraction(&obs)), reach))
            .collect::<Posterior<NlheSecret>>()
    }

    /// Reach of one complete-info history along `subject`'s decision nodes —
    /// the product of blueprint averaged policy at each node `subject` acts.
    fn reach(&self, case: Perfect, subject: NlheTurn) -> Probability {
        self.encoder
            .replay(NlheGame::from(case.root()), case.history().into_iter().map(NlheEdge::from))
            .into_iter()
            .filter(|(t, _, _)| *t == subject)
            .map(|(_, ref i, ref e)| self.blueprint.averaged_policy(i, e))
            .product()
    }
}

impl<B> Adapt<'_, B>
where
    B: CfrSolution<T = NlheTurn, E = NlheEdge, G = NlheGame, I = NlheInfo> + Sync,
{
    /// Safe subgame solve (no depth limit). Parallels [`Nlhe::adapt_safe`].
    pub fn adapt_safe(
        &self,
        recall: &Witness,
    ) -> WorldSolver<'_, { pokerkit::N_WORLDS }, B, NlheEncoder, NlheInfo, NlheSecret> {
        let (external, partition, recall) = self.setup(recall);
        WorldSolver::new(self.encoder, &self.blueprint, external, partition, recall)
    }

    /// Combined safe + depth-limited solve. Parallels [`Nlhe::adapt_full`].
    pub fn adapt_full(
        &self,
        recall: &Witness,
    ) -> SubGameSolver<'_, { pokerkit::N_WORLDS }, { pokerkit::FRONTIER_LEAVES }, Self, NlheInfo, NlheSecret> {
        let (external, partition, recall) = self.setup(recall);
        SubGameSolver::new(self, external, partition, recall)
    }

    /// Common setup for the safe solvers: external identity, belief partition
    /// from the opponent range, and the current-street recall.
    fn setup(&self, recall: &Witness) -> (NlheTurn, Belief<NlheSecret, { pokerkit::N_WORLDS }>, CfrRecall<NlheGame>) {
        let external = opposing(recall.turn());
        let partition = self.opponent_range(recall).partition();
        let path = subgame_descents(recall);
        let game = NlheGame::from(recall.head());
        (external, partition, CfrRecall::new(path, game))
    }
}

/// Current-street `(turn, edge)` descents from a witness. Duplicated from
/// `solver.rs` rather than widening its visibility, to leave the existing
/// non-async path untouched.
fn subgame_descents(recall: &Witness) -> Vec<Descent<NlheTurn, NlheEdge>> {
    recall
        .states()
        .into_iter()
        .zip(recall.history().iter())
        .map(|(state, edge)| (state.turn(), *edge))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .take_while(|(_, e)| e.is_choice())
        .map(|(t, e)| Descent(NlheTurn::from(t), NlheEdge::from(e)))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// The other seat in a two-player game. Panics on chance/terminal turns.
fn opposing(turn: Turn) -> NlheTurn {
    match turn {
        Turn::Choice(0) => NlheTurn::from(1_usize),
        Turn::Choice(1) => NlheTurn::from(0_usize),
        _ => unreachable!("two-player game requires Choice turn"),
    }
}
