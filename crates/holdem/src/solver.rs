use super::*;
use atlas::*;
use deuce::Hand;
use deuce::HandIterator;
use deuce::Hole;
use deuce::Observation;
use endgame::*;
use horizon::*;
use kicker::*;
use mccfr::*;
use pokerkit::*;

mccfr!(Nlhe, NlheEncoder, NlheTurn, NlheEdge, NlheGame, NlheInfo, 128);

/// Authoritative current-street `(turn, edge)` pairs from a witness recall.
///
/// Walks `recall.states()` alongside `recall.history()` — the state at
/// index `i` is the game BEFORE action `i`, so its turn is the turn that
/// owns that edge. Trims to trailing choice edges, matching
/// [`Recall::subgame`]'s definition of "current street." This is the
/// only safe way to get turns for a nlhe prefix: replaying edges
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

impl<R, W, S> DepthSampler<{ pokerkit::FRONTIER_LEAVES }> for Nlhe<R, W, S>
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
    ) -> Payoffs<{ pokerkit::FRONTIER_LEAVES }> {
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
    pub fn adapt_leaf(&self, recall: &Witness) -> DepthSolver<'_, Self, { pokerkit::FRONTIER_LEAVES }> {
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
    ) -> WorldSolver<'_, { pokerkit::N_WORLDS }, NlheProfile, NlheEncoder, NlheInfo, NlheSecret> {
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
    ) -> SubGameSolver<'_, { pokerkit::N_WORLDS }, { pokerkit::FRONTIER_LEAVES }, Self, NlheInfo, NlheSecret> {
        let (external, partition, recall) = self.setup(recall);
        SubGameSolver::new(self, external, partition, recall)
    }
    /// Common setup for safe solvers: external identity, belief partition, recall.
    fn setup(&self, recall: &Witness) -> (NlheTurn, Belief<NlheSecret, { pokerkit::N_WORLDS }>, CfrRecall<NlheGame>) {
        let external = opposing(recall.turn());
        let prior = self.opponent_range(recall);
        let partition = prior.partition();
        let path = subgame_descents(recall);
        let game = NlheGame::from(recall.head());
        (external, partition, CfrRecall::new(path, game))
    }
    /// Reach for one complete-info history along `subject`'s decision
    /// nodes — the product of the blueprint's averaged policy at every
    /// node where `subject` was to act. This is P(`subject`'s actions |
    /// `subject`'s hand) for the single hand encoded by `case`.
    ///
    /// The original "opponent reach" specializes this with
    /// `subject = external` (the non-`internal` player); the signalled
    /// reach specializes it with `subject = hero`.
    fn reach(&self, case: Perfect, subject: NlheTurn) -> Probability {
        self.encoder
            .replay(NlheGame::from(case.root()), case.history().into_iter().map(NlheEdge::from))
            .into_iter()
            .filter(|(t, _, _)| *t == subject)
            .map(|(_, ref i, ref e)| self.profile.averaged_policy(i, e))
            .product()
    }

    /// Unnormalized hole-card-level posterior — the raw `(Observation,
    /// reach)` stream that backs both [`Self::opponent_range`] (which
    /// projects to abstractions) and [`Self::opponent_observations`]
    /// (which normalizes and surfaces hole cards). Walks
    /// [`Witness::possibilities`] and computes external reach via
    /// `Self::reach` along the observed action sequence. This
    /// is the unnormalized P(hand | actions) ∝ P(actions | hand) × 1
    /// from a uniform prior.
    fn opponent_reaches(&self, recall: &Witness) -> Vec<(Observation, Probability)> {
        let external = opposing(recall.turn());
        recall
            .possibilities()
            .into_iter()
            .map(|(obs, case)| (obs, self.reach(case, external)))
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
    /// 2. **Likelihood** — For each external hand, `Witness::histories`
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
        normalize(self.opponent_reaches(recall))
    }

    /// Hole-card-level normalized hero **signalled** range — what the
    /// opponent's posterior over hero's hand could look like, given
    /// hero's observed action history. Mirrors
    /// [`Self::opponent_observations`] with roles swapped.
    ///
    /// Prior is uniform over `deck − board`; we don't condition on the
    /// opponent's actual hole because we don't know it. ~2 cards' worth
    /// of removal is ignored — acceptable at the 169-cell projection.
    /// The stub opponent hand passed to [`Perfect`] is semantically
    /// inert: hero's [`NlheInfo`] depends only on hero's hole + public
    /// edges, and `Self::reach` filters to hero decision nodes.
    pub fn signalled_observations(&self, recall: &Witness) -> Vec<(Observation, Probability)> {
        normalize(self.signalled_reaches(recall))
    }

    /// Unnormalized signalled-reach stream. Sibling of
    /// [`Self::opponent_reaches`] with hero/opponent roles flipped.
    fn signalled_reaches(&self, recall: &Witness) -> Vec<(Observation, Probability)> {
        let hero = NlheTurn::from(recall.turn());
        let board: Hand = recall.arr().public().into_iter().collect();
        HandIterator::from((2, board))
            .map(|hole| {
                let stub = HandIterator::from((2, Hand::add(hole, board)))
                    .next()
                    .expect("stub hole");
                let arr = Arrangement::from(hole.chain(board).collect::<Vec<_>>());
                let case = Perfect::from((&recall.replace(arr), Hole::from(stub)));
                (Observation::from((hole, board)), self.reach(case, hero))
            })
            .collect()
    }
}

/// In a 2-player game, the other seat. Panics for chance/terminal turns.
fn opposing(turn: Turn) -> NlheTurn {
    match turn {
        Turn::Choice(0) => NlheTurn::from(1_usize),
        Turn::Choice(1) => NlheTurn::from(0_usize),
        _ => unreachable!("two-player game requires Choice turn"),
    }
}

/// Normalize a `(observation, reach)` stream to sum to 1, leaving an
/// all-zero stream untouched.
fn normalize(raws: Vec<(Observation, Probability)>) -> Vec<(Observation, Probability)> {
    let total = raws.iter().map(|(_, r)| *r).sum::<Probability>();
    match total {
        0.0 => raws,
        mass => raws.into_iter().map(|(obs, r)| (obs, r / mass)).collect(),
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl<R, W, S> ledger::Hydrate for Nlhe<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        Self::new(NlheProfile::hydrate(client.clone()).await, NlheEncoder::hydrate(client.clone()).await)
    }
}
