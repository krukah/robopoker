//! [`Adapt`] — a subgame sampler generalized over its blueprint store.
//!
//! [`Flagship`] hard-codes `Blueprint = Book<NlheInfo>`, the whole strategy table in RAM. [`Adapt`]
//! offers the same `adapt_leaf` / `adapt_safe` / `adapt_full` surface over any `B` readable
//! through [`Profile`] / [`Solution`] — [`LazyBlueprint`] for DB-backed RAM-free play
//! (aliased [`Lazy`]), or [`Book<NlheInfo>`] moved in.
use super::*;
use kicker::*;
use mccfr::*;
use pokerkit::*;
use subgame::*;

/// A subgame sampler bundling a borrowed encoder with an owned blueprint `B`.
///
/// Encoding delegates to the [`NlheEncoder`]; strategy lookups go through `B`.
/// Instantiate `B` with [`LazyBlueprint`] for RAM-free play or [`Book<NlheInfo>`]
/// for the in-memory table.
pub struct Adapt<'encoder, B> {
    encoder: &'encoder NlheEncoder,
    blueprint: B,
}

/// The DB-backed instantiation: a subgame sampler that never loads the
/// blueprint into RAM.
pub type Lazy<'encoder> = Adapt<'encoder, LazyBlueprint>;

impl<'encoder, B> Adapt<'encoder, B> {
    pub fn new(encoder: &'encoder NlheEncoder, blueprint: B) -> Self {
        Self { encoder, blueprint }
    }

    pub fn encoder(&self) -> &'encoder NlheEncoder {
        self.encoder
    }

    pub fn blueprint(&self) -> &B {
        &self.blueprint
    }
}

impl Adapt<'_, LazyBlueprint> {
    /// Distinct blueprint infos pulled from the store this session — the
    /// realized working-set size, versus the whole table under [`Flagship`].
    pub fn fetches(&self) -> usize {
        self.blueprint().fetches()
    }
}

impl<B> CfrSpec for Adapt<'_, B> {
    type F = NlheInfo;
}

impl<B> Encoding for Adapt<'_, B> {
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

impl<B> DepthSampler<{ pokerkit::FRONTIER_LEAVES }> for Adapt<'_, B>
where
    B: Profile<F = NlheInfo>,
{
    type Blueprint = B;

    fn blueprint(&self) -> &Self::Blueprint {
        &self.blueprint
    }

    fn payoffs(
        &self,
        prefix: &Prefix<NlheGame>,
        game: &NlheGame,
        internal: NlheTurn,
    ) -> Payoffs<{ pokerkit::FRONTIER_LEAVES }> {
        let rollouts = FrontierHyperParams::get().rollouts();
        Payoffs::tabulate(|k, j| {
            (0..rollouts)
                .map(|_| {
                    self.encoder()
                        .biased_rollout(prefix, game, internal, k, j, self.blueprint())
                })
                .sum::<Utility>()
                / rollouts as Utility
        })
    }
}

impl<B, const W: usize> WorldRestrict<W> for Adapt<'_, B> {
    fn restrict(
        &self,
        external: TurnOf<Self>,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self> {
        self.encoder().restrict(external, world, belief, observed)
    }
}

impl<B> Adapt<'_, B>
where
    B: Profile<F = NlheInfo> + Clock + Sync,
{
    /// Depth-limited solve rooted at `recall.head()`, reading strategy from
    /// `B`. Parallels [`NlheSolver::adapt_leaf`].
    pub fn adapt_leaf(&self, recall: &Witness) -> DepthSolver<'_, Self, { pokerkit::FRONTIER_LEAVES }> {
        let internal = NlheTurn::from(recall.turn());
        let entry = NlheGame::from(recall.head());
        let prefix = NlheGrid::subgame(recall);
        DepthSolver::new(self, prefix, internal, entry)
    }

    /// Opponent posterior over abstraction buckets, from blueprint reach along
    /// the observed line. Parallels [`NlheSolver::opponent_range`].
    fn opponent_range(&self, recall: &Witness) -> Posterior<NlheSecret> {
        let external = NlheTurn::opponent(recall);
        recall
            .possibilities()
            .into_iter()
            .map(|(obs, case)| (obs, self.reach(case, external)))
            .map(|(obs, reach)| (NlheSecret::from(self.encoder().abstraction(&obs)), reach))
            .collect::<Posterior<NlheSecret>>()
    }

    /// Reach of one complete-info history along `subject`'s decision nodes —
    /// the product of blueprint averaged policy at each node `subject` acts.
    /// Retraces for the same reason [`Nlhe::reach`] does: an edge replay
    /// re-deals the board at random past a street boundary.
    fn reach(&self, case: Perfect, subject: NlheTurn) -> Probability {
        self.encoder()
            .retrace(&case)
            .into_iter()
            .filter(|choice| choice.turn() == subject)
            .map(|choice| self.blueprint().averaged_policy(choice.info(), choice.edge()))
            .product()
    }
}

impl<B> Adapt<'_, B>
where
    B: Solution<F = NlheInfo> + Sync,
{
    /// Safe subgame solve (no depth limit). Parallels [`NlheSolver::adapt_safe`].
    pub fn adapt_safe(&self, recall: &Witness) -> WorldSolver<'_, { pokerkit::N_WORLDS }, B, NlheEncoder> {
        let (external, partition, recall) = self.setup(recall);
        WorldSolver::new(self.encoder(), self.blueprint(), external, partition, recall)
    }

    /// Combined safe + depth-limited solve. Parallels [`NlheSolver::adapt_full`].
    pub fn adapt_full(
        &self,
        recall: &Witness,
    ) -> SubGameSolver<'_, { pokerkit::N_WORLDS }, { pokerkit::FRONTIER_LEAVES }, Self> {
        let (external, partition, recall) = self.setup(recall);
        SubGameSolver::new(self, external, partition, recall)
    }

    /// Common setup for the safe solvers: external identity, belief partition
    /// from the opponent range, and the current-street recall.
    fn setup(&self, recall: &Witness) -> (NlheTurn, Belief<NlheSecret, { pokerkit::N_WORLDS }>, CfrRecall<NlheGame>) {
        let external = NlheTurn::opponent(recall);
        let partition = self.opponent_range(recall).partition();
        let path = NlheGrid::subgame(recall);
        let game = NlheGame::from(recall.head());
        (external, partition, CfrRecall::new(path, game))
    }
}
