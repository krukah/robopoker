use super::*;
use rbp_cards::Observation;
use rbp_core::*;
use rbp_depth::*;
use rbp_gameplay::*;
use rbp_mccfr::*;
use rbp_subgame::*;
use rbp_world::*;

mccfr!(
    Nlhe,
    NlheEncoder,
    NlheTurn,
    NlheEdge,
    NlheGame,
    NlheInfo,
    128
);

/// Authoritative current-street `(turn, edge)` pairs from a witness recall.
///
/// Walks `recall.states()` alongside `recall.history()` — the state at
/// index `i` is the game BEFORE action `i`, so its turn is the turn that
/// owns that edge. Trims to trailing choice edges, matching
/// [`Recall::subgame`]'s definition of "current street." This is the
/// only safe way to get turns for a holdem prefix: replaying edges
/// from `NlheGame::root()` would silently diverge at chip snapping or
/// chance card draws.
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

impl<R, W, S> DepthSampler<{ rbp_core::FRONTIER_LEAVES }> for Nlhe<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type Blueprint = NlheProfile;

    fn blueprint(&self) -> &Self::Blueprint {
        &self.profile
    }

    fn payoffs(
        &self,
        prefix: &Prefix<NlheTurn, NlheEdge>,
        game: &NlheGame,
        internal: NlheTurn,
    ) -> Payoffs<{ rbp_core::FRONTIER_LEAVES }> {
        let ref encoder = self.encoder;
        let ref profile = self.profile;
        let rollouts = FrontierHyperParams::get().rollouts();
        Payoffs::tabulate(|k, j| {
            (0..rollouts)
                .map(|_| encoder.biased_rollout(prefix, game, internal, k, j, profile))
                .sum::<Utility>()
                / rollouts as Utility
        })
    }
}

impl<R, W, S, const WORLDS: usize> WorldRestrict<WORLDS> for Nlhe<R, W, S>
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
        self.encoder.restrict(external, world, belief, observed)
    }
}

impl<R, W, S> Nlhe<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    /// Creates a depth-limited solver rooted at the current decision point.
    ///
    /// No opponent range partitioning or world sampling — solves the
    /// depth-limited tree from `recall.head()` using biased continuation
    /// rollouts at the leaves.
    pub fn adapt_leaf(
        &self,
        recall: &Witness,
    ) -> DepthSolver<'_, Self, { rbp_core::FRONTIER_LEAVES }> {
        let internal = NlheTurn::from(recall.turn());
        let entry = NlheGame::from(recall.head());
        let prefix = subgame_descents(recall);
        DepthSolver::new(self, prefix, internal, entry)
    }
    /// Creates a safe subgame solver from game history (no depth limiting).
    ///
    /// Identical setup to [`Self::adapt_full`] but uses [`WorldSolver`]
    /// which expands the full tree to terminal nodes instead of cutting
    /// off at a depth limit.
    pub fn adapt_safe(
        &self,
        recall: &Witness,
    ) -> WorldSolver<
        '_,
        { rbp_core::N_WORLDS },
        NlheProfile,
        NlheEncoder,
        NlheInfo,
        NlheSecret,
    > {
        let (external, partition, recall) = self.setup(recall);
        WorldSolver::new(&self.encoder, &self.profile, external, partition, recall)
    }
    /// Creates a combined safe + depth-limited subgame solver.
    ///
    /// Computes opponent reach distribution, partitions into K worlds
    /// with secret-to-world mapping, and initializes the solver from
    /// game root through the prefix. The partition enables rejection
    /// sampling of card deals to condition on the selected world.
    pub fn adapt_full(
        &self,
        recall: &Witness,
    ) -> SubGameSolver<
        '_,
        { rbp_core::N_WORLDS },
        { rbp_core::FRONTIER_LEAVES },
        Self,
        NlheInfo,
        NlheSecret,
    > {
        let (external, partition, recall) = self.setup(recall);
        SubGameSolver::new(self, external, partition, recall)
    }
    /// Common setup for safe solvers: external identity, belief partition, recall.
    fn setup(
        &self,
        recall: &Witness,
    ) -> (
        NlheTurn,
        Belief<NlheSecret, { rbp_core::N_WORLDS }>,
        CfrRecall<NlheGame>,
    ) {
        let external = match recall.turn() {
            Turn::Choice(0) => NlheTurn::from(1),
            Turn::Choice(1) => NlheTurn::from(0),
            _ => unreachable!("subgame solving requires two-player game"),
        };
        let prior = self.opponent_range(recall);
        let partition = prior.partition();
        let path = subgame_descents(recall);
        let game = NlheGame::from(recall.head());
        (external, partition, CfrRecall::new(path, game))
    }
    /// External reach for one complete-info history — the product of the
    /// blueprint's averaged policy at every external decision node along
    /// `case`'s action sequence. This is P(actions | hand) for the single
    /// hand encoded by `case`.
    fn external_reach(&self, case: Perfect, internal: NlheTurn) -> Probability {
        self.encoder
            .replay(
                NlheGame::from(case.root()),
                case.history().into_iter().map(NlheEdge::from),
            )
            .into_iter()
            .filter(|(t, _, _)| t.is_opponent(&internal))
            .map(|(_, ref i, ref e)| self.profile.averaged_policy(i, e))
            .product()
    }

    /// Unnormalized hole-card-level posterior — the raw `(Observation,
    /// reach)` stream that backs both [`Self::opponent_range`] (which
    /// projects to abstractions) and [`Self::opponent_observations`]
    /// (which normalizes and surfaces hole cards). Walks
    /// [`Witness::possibilities`] and computes external reach via
    /// [`Self::external_reach`] along the observed action sequence. This
    /// is the unnormalized P(hand | actions) ∝ P(actions | hand) × 1
    /// from a uniform prior.
    fn opponent_reaches(&self, recall: &Witness) -> Vec<(Observation, Probability)> {
        let internal = NlheTurn::from(recall.turn());
        recall
            .possibilities()
            .into_iter()
            .map(|(obs, case)| (obs, self.external_reach(case, internal)))
            .collect()
    }
    /// Computes the opponent's posterior range over abstraction buckets.
    ///
    /// Transforms priors into posteriors through four layers:
    ///
    /// 1. **Uniform prior** — [`Observation::opponents`] enumerates all
    ///    external hole cards consistent with internal's information
    ///    (excludes internal pocket and dealt board cards), each with
    ///    implicit weight 1.
    ///
    /// 2. **Likelihood** — For each external hand, [`Witness::histories`]
    ///    builds a complete-info [`Perfect`] history, and
    ///    [`Solver::external_reach`] computes the product of external's
    ///    blueprint action probabilities at every external decision node
    ///    along the observed action sequence. This is P(actions | hand).
    ///
    /// 3. **Unnormalized posterior** — Each (observation, reach) pair is the
    ///    unnormalized P(hand | actions) ∝ P(actions | hand) × 1.
    ///
    /// 4. **Abstraction projection** — Observations are mapped to abstraction
    ///    buckets via the encoder; reach values sharing a bucket are summed.
    ///    The result is a `Posterior<NlheSecret>` suitable for partitioning
    ///    into subgame worlds.
    pub fn opponent_range(&self, recall: &Witness) -> Posterior<NlheSecret> {
        self.opponent_reaches(recall)
            .into_iter()
            .map(|(obs, reach)| (NlheSecret::from(self.encoder.abstraction(&obs)), reach))
            .collect::<Posterior<NlheSecret>>()
    }
    /// Hole-card-level normalized opponent range.
    ///
    /// Same likelihood computation as [`Self::opponent_range`] but skips
    /// the abstraction projection — yields one entry per concrete villain
    /// hole-card combo with weights normalized to sum to 1. Intended for
    /// surfacing the opponent's range to clients at the granularity the
    /// frontend cares about (hole cards), not the granularity CFR
    /// solves at (abstraction buckets).
    pub fn opponent_observations(&self, recall: &Witness) -> Vec<(Observation, Probability)> {
        let raws = self.opponent_reaches(recall);
        let total = raws.iter().map(|(_, r)| *r).sum::<Probability>();
        match total {
            0.0 => raws,
            mass => raws.into_iter().map(|(obs, r)| (obs, r / mass)).collect(),
        }
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl<R, W, S> rbp_database::Hydrate for Nlhe<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        Self::new(
            NlheProfile::hydrate(client.clone()).await,
            NlheEncoder::hydrate(client.clone()).await,
        )
    }
}
