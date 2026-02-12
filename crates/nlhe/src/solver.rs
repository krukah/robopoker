use super::*;
use rbp_core::Utility;
use rbp_gameplay::*;
use rbp_mccfr::*;
use rbp_mccfr::Posterior;
use std::collections::BTreeMap;
use std::marker::PhantomData;

// TODO: Import from rbp-core or define locally
const SUBGAME_ITERATIONS: usize = 100;
const CFR_TREE_COUNT_NLHE: usize = 1;
const CFR_BATCH_SIZE_NLHE: usize = 1000;

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
    ) -> SubSolver<'_, NlheProfile, NlheEncoder, SUBGAME_ITERATIONS> {
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
        CFR_BATCH_SIZE_NLHE
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
