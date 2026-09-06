use super::*;
use deuce::Hand;
use deuce::HandIterator;
use deuce::Hole;
use deuce::Observation;
use kicker::*;
use mccfr::*;
use pokerkit::*;
use subgame::*;

mccfr!(Nlhe, NlheEncoder, NlheTurn, NlheEdge, NlheGame, NlheInfo, 128);

/// Authoritative current-street `(turn, edge)` pairs from a witness recall.
///
/// The state at index `i` is the game BEFORE action `i`, so its turn owns that
/// edge. Trims to trailing choice edges, matching [`Recall::subgame`]'s notion
/// of "current street." This is the only safe way to get turns for a nlhe
/// prefix: replaying edges from `NlheGame::root()` would silently diverge at
/// chip snapping or chance card draws.
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
    /// Depth-limited solver rooted at the current decision point: no opponent
    /// range partitioning or world sampling, biased rollouts at the leaves.
    pub fn adapt_leaf(&self, recall: &Witness) -> DepthSolver<'_, Self, { pokerkit::FRONTIER_LEAVES }> {
        let internal = NlheTurn::from(recall.turn());
        let entry = NlheGame::from(recall.head());
        let prefix = subgame_descents(recall);
        DepthSolver::new(self, prefix, internal, entry)
    }
    /// Safe subgame solver: [`Self::adapt_full`]'s setup, but [`WorldSolver`]
    /// expands to terminal nodes instead of cutting off at a depth limit.
    pub fn adapt_safe(
        &self,
        recall: &Witness,
    ) -> WorldSolver<'_, { pokerkit::N_WORLDS }, NlheProfile, NlheEncoder, NlheInfo, NlheSecret> {
        let (external, partition, recall) = self.setup(recall);
        WorldSolver::new(&self.encoder, &self.profile, external, partition, recall)
    }
    /// Combined safe + depth-limited subgame solver. The opponent posterior is
    /// partitioned into K worlds; the partition drives rejection sampling of
    /// card deals so the tree conditions on the selected world.
    pub fn adapt_full(
        &self,
        recall: &Witness,
    ) -> SubGameSolver<'_, { pokerkit::N_WORLDS }, { pokerkit::FRONTIER_LEAVES }, Self, NlheInfo, NlheSecret> {
        let (external, partition, recall) = self.setup(recall);
        SubGameSolver::new(self, external, partition, recall)
    }
    /// Modicum-style nested subgame solver for an off-tree raise.
    ///
    /// Like [`Self::adapt_full`], but over [`Nest`] wrapper types: the entry
    /// node's menu is augmented with the literal `offtree` amount, and CFR
    /// jointly resolves mixing over the canonical sizes plus that literal.
    /// `recall` must be rolled back to *before* the off-tree action, so it
    /// becomes a menu option at the entry rather than history.
    ///
    /// Requires `&'static self` (the live player holds a `&'static Flagship`)
    /// because the `Nest` source must outlive the returned solver; it is
    /// `Box::leak`ed — a bounded leak of one source per off-tree re-solve.
    pub fn adapt_nested(
        &'static self,
        recall: &Witness,
        offtree: Action,
    ) -> SubGameSolver<
        'static,
        { pokerkit::N_WORLDS },
        { pokerkit::FRONTIER_LEAVES },
        Nest<'static, R, W, S>,
        NestInfo<NlheInfo, Action>,
        NlheSecret,
    > {
        let (external, partition, recall) = self.setup(recall);
        let recall = CfrRecall::new(
            recall
                .descents()
                .iter()
                .map(|d| Descent(d.0, NestEdge::Game(d.1)))
                .collect::<Vec<_>>(),
            NestGame::entry(recall.game(), offtree),
        );
        SubGameSolver::new(Box::leak(Box::new(Nest::new(self, offtree))), external, partition, recall)
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

    /// Unnormalized hole-card-level posterior backing both
    /// [`Self::opponent_range`] and [`Self::opponent_observations`]: over a
    /// uniform prior on [`Witness::possibilities`], reach is the unnormalized
    /// P(hand | actions) ∝ P(actions | hand).
    fn opponent_reaches(&self, recall: &Witness) -> Vec<(Observation, Probability)> {
        let external = opposing(recall.turn());
        recall
            .possibilities()
            .into_iter()
            .map(|(obs, case)| (obs, self.reach(case, external)))
            .collect()
    }
    /// [`Self::opponent_reaches`] projected onto abstraction buckets, summing
    /// reach within a bucket — the granularity subgame worlds partition at.
    pub fn opponent_range(&self, recall: &Witness) -> Posterior<NlheSecret> {
        self.opponent_reaches(recall)
            .into_iter()
            .map(|(obs, reach)| (NlheSecret::from(self.encoder.abstraction(&obs)), reach))
            .collect::<Posterior<NlheSecret>>()
    }
    /// [`Self::opponent_range`] without the abstraction projection: one
    /// normalized entry per concrete villain combo. For clients, which care
    /// about hole cards rather than the buckets CFR solves at.
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
impl<R, W, S> daybook::Hydrate for Nlhe<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        Self::new(NlheProfile::hydrate(client.clone()).await, NlheEncoder::hydrate(client.clone()).await)
    }
}
