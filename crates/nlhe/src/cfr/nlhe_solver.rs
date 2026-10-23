use super::*;
use deuce::Hand;
use deuce::HandIterator;
use deuce::Hole;
use deuce::Observation;
use kicker::*;
use mccfr::*;
use pokerkit::*;
use std::collections::BTreeMap;
use subgame::*;

/// Hold'em queries and subgame solvers over a blueprint: range posteriors, per-hand strategies,
/// and the depth-limited, safe, combined and nested re-solvers rooted at a live decision.
pub trait NlheSolver:
    Solver<F = NlheInfo, N = NlheEncoder, P = Book<NlheInfo>>
    + DepthSampler<{ pokerkit::FRONTIER_LEAVES }, Blueprint = Book<NlheInfo>>
    + WorldRestrict<{ pokerkit::N_WORLDS }>
{
    /// Modicum-style nested subgame solver for an off-tree raise.
    ///
    /// Like [`Self::adapt_full`], over [`Nest`] wrapper types: the entry node's menu gains the
    /// literal `offtree` amount. `recall` must be rolled back to *before* the off-tree action.
    fn adapt_nested(
        &'static self,
        recall: &Witness,
        offtree: Action,
    ) -> SubGameSolver<
        'static,
        { pokerkit::N_WORLDS },
        { pokerkit::FRONTIER_LEAVES },
        Nest<'static, Self::R, Self::W, Self::S>,
    >
    where
        Self: 'static;

    /// Depth-limited solver rooted at the current decision point: no opponent
    /// range partitioning or world sampling, biased rollouts at the leaves.
    fn adapt_leaf(&self, recall: &Witness) -> DepthSolver<'_, Self, { pokerkit::FRONTIER_LEAVES }> {
        let internal = NlheTurn::from(recall.turn());
        let entry = NlheGame::from(recall.head());
        let prefix = NlheGrid::subgame(recall);
        DepthSolver::new(self, prefix, internal, entry)
    }
    /// Safe subgame solver: [`Self::adapt_full`]'s setup, but [`WorldSolver`]
    /// expands to terminal nodes instead of cutting off at a depth limit.
    fn adapt_safe(&self, recall: &Witness) -> WorldSolver<'_, { pokerkit::N_WORLDS }, Book<NlheInfo>, NlheEncoder> {
        let (external, partition, recall) = self.setup(recall);
        WorldSolver::new(self.encoder(), self.profile(), external, partition, recall)
    }
    /// Combined safe + depth-limited subgame solver. The opponent posterior is
    /// partitioned into K worlds; the partition drives rejection sampling of
    /// card deals so the tree conditions on the selected world.
    fn adapt_full(
        &self,
        recall: &Witness,
    ) -> SubGameSolver<'_, { pokerkit::N_WORLDS }, { pokerkit::FRONTIER_LEAVES }, Self> {
        let (external, partition, recall) = self.setup(recall);
        SubGameSolver::new(self, external, partition, recall)
    }
    /// Common setup for safe solvers: external identity, belief partition, recall.
    fn setup(&self, recall: &Witness) -> (NlheTurn, Belief<NlheSecret, { pokerkit::N_WORLDS }>, CfrRecall<NlheGame>) {
        let external = NlheTurn::opponent(recall);
        let prior = self.opponent_range(recall);
        let partition = prior.partition();
        let path = NlheGrid::subgame(recall);
        let game = NlheGame::from(recall.head());
        (external, partition, CfrRecall::new(path, game))
    }
    /// Reach for one complete-info history along `subject`'s decision nodes — the product of
    /// the blueprint's averaged policy at every node where `subject` was to act.
    fn reach(&self, case: Perfect, subject: NlheTurn) -> Probability {
        self.encoder()
            .retrace(&case)
            .into_iter()
            .filter(|choice| choice.turn() == subject)
            .map(|choice| self.profile().averaged_policy(choice.info(), choice.edge()))
            .product()
    }

    /// How many of `subject`'s own decisions `reach` multiplies
    /// over. Card-independent — the action history is fixed — so a single
    /// count describes the whole posterior. Zero means every candidate hand
    /// got an empty product, and [`Self::observations`] is reporting the
    /// uniform prior over `deck − board`, not anything the blueprint learned.
    fn decisions(&self, recall: &Witness, subject: NlheTurn) -> usize {
        recall
            .plays()
            .into_iter()
            .filter(|(seat, _, _)| NlheTurn::from(*seat) == subject)
            .count()
    }

    /// Blueprint policy for every concrete private holding still compatible
    /// with the public board. A client may then fold suit-distinct combos
    /// into its 169-cell range grid without erasing board-specific strategy.
    fn strategies(&self, recall: &Witness) -> Vec<(Observation, BTreeMap<Edge, Probability>)> {
        let board = recall.arr().public().into_iter().collect::<Hand>();
        HandIterator::from((2, board))
            .map(|hole| Observation::from((hole, board)))
            .map(|obs| {
                let info = NlheInfo::from((recall, self.encoder().abstraction(&obs)));
                let policy = info
                    .choices()
                    .into_iter()
                    .map(|edge| (edge, self.profile().averaged_policy(&info, &NlheEdge::from(edge))))
                    .collect();
                (obs, policy)
            })
            .collect()
    }

    /// Unnormalized posterior over `subject`'s hole cards: reach is the unnormalized P(hand |
    /// actions) ∝ P(actions | hand) for each candidate `hole` in `holes`.
    fn reaches(
        &self,
        recall: &Witness,
        subject: NlheTurn,
        holes: impl Iterator<Item = Hand>,
    ) -> Vec<(Observation, Probability)> {
        let board = recall.arr().public().into_iter().collect::<Hand>();
        holes
            .map(|hole| {
                (
                    Observation::from((hole, board)),
                    self.reach(Self::naive_case(recall, subject, hole, board), subject),
                )
            })
            .collect()
    }

    /// The complete-info world [`Self::reaches`] scores for one candidate
    /// `hole`: `subject` holds `hole`, the off-subject seat holds an inert
    /// `stub`. The stub only removes two cards — every info keys on the acting
    /// seat's own cards, so it can never reach a policy lookup — so which two
    /// it removes is irrelevant, and one naive world-builder serves every prior.
    fn naive_case(recall: &Witness, subject: NlheTurn, hole: Hand, board: Hand) -> Perfect {
        let stub = HandIterator::from((2, Hand::add(hole, board)))
            .next()
            .expect("stub hole");
        let (pov, foe) = if NlheTurn::narrator(recall) == subject { (hole, stub) } else { (stub, hole) };
        Perfect::from((&recall.replace(Arrangement::from(pov.chain(board).collect::<Vec<_>>())), Hole::from(foe)))
    }

    /// Normalized hole-card-level posterior over `subject`'s hand — the shared body of
    /// `/strategy/range` ([`NlheTurn::opponent`]) and `/strategy/signalled`
    /// ([`NlheTurn::narrator`]). Clients care about hole cards rather than the buckets CFR
    /// solves at, so there is no projection.
    fn observations(&self, recall: &Witness, subject: NlheTurn) -> Vec<(Observation, Probability)> {
        Solving::normalize(self.reaches(
            recall,
            subject,
            HandIterator::from((2, recall.arr().public().into_iter().collect())),
        ))
    }

    /// Opponent posterior over abstraction buckets for subgame solving, **conditioned on hero's
    /// actual hole** — a world in which villain holds hero's cards is impossible, not merely
    /// unlikely. The only difference from the spectator [`Self::observations`] is the prior:
    /// candidate holes are drawn from `deck − board − hero` (the [`Observation::opponents`]
    /// dead-set), not the full `deck − board`. Feeds `setup`; never the range panels.
    fn opponent_range(&self, recall: &Witness) -> Posterior<NlheSecret> {
        self.reaches(recall, NlheTurn::opponent(recall), HandIterator::from((2, Hand::from(recall.seen()))))
            .into_iter()
            .map(|(obs, reach)| (NlheSecret::from(self.encoder().abstraction(&obs)), reach))
            .collect::<Posterior<NlheSecret>>()
    }
}

impl<R, W, S> NlheSolver for Tabular<NlheEncoder, R, W, S, 128>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn adapt_nested(
        &'static self,
        recall: &Witness,
        offtree: Action,
    ) -> SubGameSolver<'static, { pokerkit::N_WORLDS }, { pokerkit::FRONTIER_LEAVES }, Nest<'static, R, W, S>>
    where
        Self: 'static,
    {
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
}
