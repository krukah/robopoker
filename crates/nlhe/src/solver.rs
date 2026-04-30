use super::*;
use rand::SeedableRng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rbp_core::Utility;
use rbp_gameplay::*;
use rbp_mccfr::Posterior;
use rbp_mccfr::*;
use rbp_transport::Density;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::sync::Arc;

// TODO: Import from rbp-core or define locally
const SUBGAME_ITERATIONS: usize = 100;
const CFR_TREE_COUNT_NLHE: usize = 1;
const CFR_BATCH_SIZE_NLHE: usize = 1000;
const CFR_BATCH_TREES_PER_THREAD: usize = 8;
const DLS_ROLLOUTS_PER_LEAF: usize = 8;
const DLS_MAX_ROLLOUT_ACTIONS: usize = 128;
pub const DLS_MAX_SOLVE_MS: u64 = 5_000;

/// Runtime settings for heads-up depth-limited solving.
#[derive(Clone, Copy, Debug)]
pub struct DlsOptions {
    pub rollouts_per_leaf: usize,
    pub max_rollout_actions: usize,
    pub max_solve_ms: u64,
}

impl Default for DlsOptions {
    fn default() -> Self {
        Self {
            rollouts_per_leaf: DLS_ROLLOUTS_PER_LEAF,
            max_rollout_actions: DLS_MAX_ROLLOUT_ACTIONS,
            max_solve_ms: DLS_MAX_SOLVE_MS,
        }
    }
}

/// Detects actions that were snapped into the action abstraction.
pub fn has_offtree_actions(recall: &impl Recall) -> bool {
    let mut game = recall.root();
    let mut path = Path::default();
    for action in recall.actions().iter().copied() {
        let edge = game.edgify(action, path.aggression());
        let canonical = game.snap(game.actionize(edge));
        if action.is_choice() && canonical != action {
            return true;
        }
        path = path
            .into_iter()
            .chain(std::iter::once(edge))
            .collect::<Path>();
        game = game.apply(action);
    }
    false
}

/// Complete MCCFR solver and trained blueprint for No-Limit Hold'em.
///
/// Combines an [`NlheEncoder`] (for state→info mapping) with an [`NlheProfile`]
/// (for regret/strategy storage) to form both a trainable [`Solver`] and an
/// inference-ready blueprint for gameplay.
///
/// # Type Parameters
///
/// - `R` — [`RegretSchedule`] for regret accumulation/discounting
/// - `W` — [`PolicySchedule`] for strategy weight accumulation
/// - `S` — [`SamplingScheme`] for tree exploration
///
/// # Training (Solver trait)
///
/// Training loop:
/// 1. Generate sampled game trees via the encoder
/// 2. Compute counterfactual values using reach probabilities
/// 3. Update regrets and strategy weights in the profile
/// 4. Repeat, alternating the traversing player each iteration
///
/// # Inference (Blueprint methods)
///
/// After training, use [`Self::subgame`] to create a [`SubSolver`] for real-time
/// refinement, or query strategies directly via the profile.
///
/// # Database Integration
///
/// With `database` feature, loads encoder abstractions and profile state
/// from PostgreSQL to resume training or serve inference requests.
pub struct NlheSolver<R, W, S>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    /// Encoder for mapping game states to information sets.
    pub encoder: NlheEncoder,
    /// Profile storing accumulated regrets and strategies.
    pub profile: NlheProfile,
    /// Phantom data for algorithm configuration.
    phantom: PhantomData<fn() -> (R, W, S)>,
}

impl<R, W, S> NlheSolver<R, W, S>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    /// Creates a new solver from profile and encoder.
    pub fn new(profile: NlheProfile, encoder: NlheEncoder) -> Self {
        Self {
            profile,
            encoder,
            phantom: PhantomData,
        }
    }
    /// Creates a subgame solver from game history.
    ///
    /// Computes opponent reach distribution, clusters into K worlds,
    /// and initializes the solver from game root through the prefix.
    pub fn subgame(
        &self,
        recall: &Partial,
    ) -> SubSolver<'_, NlheProfile, NlheEncoder, ExternalSampling, SUBGAME_ITERATIONS> {
        SubSolver::new(
            &self.encoder,
            &self.profile,
            match recall.turn() {
                Turn::Choice(0) => NlheTurn::from(1),
                Turn::Choice(1) => NlheTurn::from(0),
                _ => unreachable!("subgame solving requires two-player game...for now"),
            },
            recall.subgame().into_iter().map(NlheEdge::from).collect(),
            ManyWorlds::cluster(self.opponent_range(recall)),
        )
    }
    /// Creates a heads-up depth-limited subgame solver from game history.
    ///
    /// The solver replays the current-street prefix, stops before the next
    /// chance/street transition, and lets the opponent choose among fixed
    /// blueprint-derived continuation strategies at the frontier.
    pub fn depth_limited_subgame(
        &self,
        recall: &Partial,
    ) -> SubSolver<'_, NlheProfile, NlheEncoder, ExternalSampling, SUBGAME_ITERATIONS> {
        self.depth_limited_subgame_with(recall, DlsOptions::default())
    }

    pub fn depth_limited_subgame_with(
        &self,
        recall: &Partial,
        options: DlsOptions,
    ) -> SubSolver<'_, NlheProfile, NlheEncoder, ExternalSampling, SUBGAME_ITERATIONS> {
        let prefix = recall
            .subgame()
            .into_iter()
            .map(NlheEdge::from)
            .collect::<Vec<_>>();
        let root = Self::current_round_root(recall);
        let evaluator = Arc::new(NlheRolloutEvaluator::new(&self.encoder, options));
        SubSolver::depth_limited(
            &self.encoder,
            &self.profile,
            match recall.turn() {
                Turn::Choice(0) => NlheTurn::from(1),
                Turn::Choice(1) => NlheTurn::from(0),
                _ => unreachable!("depth-limited solving requires two-player action"),
            },
            prefix,
            ManyWorlds::cluster(self.opponent_range(recall)),
            root,
            Some(evaluator),
        )
    }

    /// Returns the public game state at the start of the current betting round.
    fn current_round_root(recall: &Partial) -> NlheGame {
        let mut game = recall.root();
        let mut root = game;
        for action in recall.actions().iter().copied() {
            game = game.apply(action);
            if action.is_chance() {
                root = game;
            }
        }
        NlheGame::from(root)
    }

    /// Computes opponent observation-level range from game history.
    ///
    /// For each possible opponent observation, constructs a [`Perfect`](crate::gameplay::Perfect)
    /// (complete info) and computes its reach probability via [`Solver::external_reach`].
    ///
    /// Returns a distribution since `Partial` has partial information —
    /// we must iterate over all possible opponent hands.
    ///
    /// Projects observation-level range to abstraction level.
    /// Aggregates reach by abstraction bucket for clustering into worlds.
    pub fn opponent_range(&self, recall: &Partial) -> Posterior<NlheSecret> {
        let hero = NlheTurn::from(recall.turn());
        recall
            .histories()
            .into_iter()
            .map(|(obs, hist)| (obs, hist.root(), hist.history().into_iter()))
            .map(|(obs, root, path)| (obs, NlheGame::from(root), path.map(NlheEdge::from)))
            .map(|(obs, root, path)| (obs, self.external_reach(root, hero, path)))
            .map(|(obs, reach)| (NlheSecret::from(self.encoder.abstraction(&obs)), reach))
            .collect::<Posterior<NlheSecret>>()
    }
}

struct NlheRolloutEvaluator<'a> {
    encoder: &'a NlheEncoder,
    options: DlsOptions,
}

impl<'a> NlheRolloutEvaluator<'a> {
    fn new(encoder: &'a NlheEncoder, options: DlsOptions) -> Self {
        Self { encoder, options }
    }

    fn rollout(
        &self,
        blueprint: &NlheProfile,
        frontier: NlheInfo,
        game: NlheGame,
        payoff_turn: NlheTurn,
        continuation: Continuation,
        rollout: usize,
    ) -> Option<Utility> {
        let mut game = Game::from(game);
        let mut path = frontier
            .history()
            .into_iter()
            .map(Edge::from)
            .collect::<Path>();
        let mut rng = self.rng(frontier, continuation, rollout);
        for _ in 0..self.options.max_rollout_actions {
            match game.turn() {
                Turn::Terminal => {
                    let player = Turn::from(payoff_turn).position();
                    return game
                        .settlements()
                        .get(player)
                        .map(|settlement| settlement.won() as Utility);
                }
                Turn::Chance => {
                    let action = game.legal().into_iter().next()?;
                    self.advance(&mut game, &mut path, action);
                }
                Turn::Choice(_) => {
                    let info = self.info(&game, path)?;
                    let policy = continuation.policy(blueprint, &info);
                    let edge = self.sample(policy, &mut rng)?;
                    let action = game.snap(game.actionize(Edge::from(edge)));
                    self.advance(&mut game, &mut path, action);
                }
            }
        }
        None
    }

    fn info(&self, game: &Game, path: Path) -> Option<NlheInfo> {
        let abstraction = self.encoder.try_abstraction(&game.sweat())?;
        let choices = game.choices(path.aggression());
        Some(NlheInfo::from((path, abstraction, choices)))
    }

    fn sample(&self, policy: Policy<NlheEdge>, rng: &mut rand::rngs::SmallRng) -> Option<NlheEdge> {
        let edges = policy.support().collect::<Vec<_>>();
        let weights = edges
            .iter()
            .map(|edge| policy.density(edge))
            .collect::<Vec<_>>();
        let index = WeightedIndex::new(&weights).ok()?.sample(rng);
        edges.get(index).copied()
    }

    fn advance(&self, game: &mut Game, path: &mut Path, action: Action) {
        let edge = game.edgify(action, path.aggression());
        *path = path
            .clone()
            .into_iter()
            .chain(std::iter::once(edge))
            .collect();
        *game = game.apply(action);
    }

    fn rng(
        &self,
        frontier: NlheInfo,
        continuation: Continuation,
        rollout: usize,
    ) -> rand::rngs::SmallRng {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        frontier.hash(&mut hasher);
        continuation.hash(&mut hasher);
        rollout.hash(&mut hasher);
        rand::rngs::SmallRng::seed_from_u64(hasher.finish())
    }
}

impl FrontierEvaluator<NlheProfile> for NlheRolloutEvaluator<'_> {
    fn evaluate(
        &self,
        blueprint: &NlheProfile,
        info: &NlheInfo,
        game: &NlheGame,
        payoff_turn: NlheTurn,
        continuation: Continuation,
    ) -> Option<Utility> {
        let rollouts = self.options.rollouts_per_leaf.max(1);
        let mut sum = 0.0;
        let mut count = 0;
        for rollout in 0..rollouts {
            if let Some(value) =
                self.rollout(blueprint, *info, *game, payoff_turn, continuation, rollout)
            {
                sum += value;
                count += 1;
            }
        }
        (count > 0).then(|| sum / count as Utility)
    }
}

impl<R, W, S> Solver for NlheSolver<R, W, S>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    type T = NlheTurn;
    type E = NlheEdge;
    type G = NlheGame;
    type I = NlheInfo;
    type X = NlhePublic;
    type Y = NlheSecret;
    type N = NlheEncoder;
    type P = NlheProfile;
    type S = S;
    type R = R;
    type W = W;

    fn tree_count() -> usize {
        CFR_TREE_COUNT_NLHE
    }
    fn batch_size() -> usize {
        let cores = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        CFR_BATCH_SIZE_NLHE.max(cores * CFR_BATCH_TREES_PER_THREAD)
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
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| Encounter::from_tuple(edge.default_policy(), edge.default_regret()))
            .weight
    }
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| Encounter::from_tuple(edge.default_policy(), edge.default_regret()))
            .regret
    }
    fn mut_evalue(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| Encounter::from_tuple(edge.default_policy(), edge.default_regret()))
            .evalue
    }
    fn mut_counts(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| Encounter::from_tuple(edge.default_policy(), edge.default_regret()))
            .counts
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl<R, W, S> rbp_database::Hydrate for NlheSolver<R, W, S>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        Self {
            encoder: NlheEncoder::hydrate(client.clone()).await,
            profile: NlheProfile::hydrate(client.clone()).await,
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_cards::Street;

    #[test]
    fn current_round_root_is_hand_root_before_first_draw() {
        let recall = Partial::initial(Turn::Choice(0)).push(Action::Call(1));
        let root = Game::from(
            NlheSolver::<LinearRegret, LinearWeight, ExternalSampling>::current_round_root(&recall),
        );
        assert_eq!(root.street(), Street::Pref);
        assert_eq!(root, recall.root());
    }

    #[test]
    fn canonical_line_is_not_offtree() {
        let recall = Partial::initial(Turn::Choice(0)).push(Action::Call(1));
        assert!(!has_offtree_actions(&recall));
    }
}
