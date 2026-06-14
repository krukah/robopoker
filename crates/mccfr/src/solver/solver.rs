use crate::*;

/// The core training orchestrator for Monte Carlo CFR.
///
/// Given access to a [`CfrSolution`] and [`CfrEncoder`], the `Solver` trait encapsulates:
/// 1. Sampling game trees via [`SamplingScheme`]
/// 2. Computing [`Decisions`] regret vectors at each [`InfoSet`]
/// 3. Updating the [`CfrSolution`] after each batch using [`RegretSchedule`] and [`WeightSchedule`]
///
/// # Associated Types
///
/// The solver bundles all types defining an extensive-form game:
///
/// - **`T: CfrTurn`** — Player/chance/terminal node classification
/// - **`E: CfrEdge`** — Actions available at decision points
/// - **`G: CfrGame`** — State transitions and payoff evaluation
/// - **`I: CfrInfo`** — Information sets combining public and secret state
/// - **`X: CfrPublic`** — Observable state (action history, board cards)
/// - **`Y: CfrSecret`** — Private state (hole cards, hand abstractions)
///
/// Plus three **algorithm variant** types that control CFR behavior:
///
/// - **`S: SamplingScheme`** — How branches are selected during tree traversal
///   (external sampling, targeted, uniform)
/// - **`R: RegretSchedule`** — How regrets are accumulated and discounted
///   (vanilla, CFR+, discounted, linear)
/// - **`W: WeightSchedule`** — How strategy weights are accumulated across iterations
///   (constant, linear, quadratic, exponential)
///
/// # Implementation
///
/// To implement a solver for a new game:
/// 1. Define game types (Turn, Edge, Game, Info, Public, Secret)
/// 2. Implement [`CfrEncoder`] and [`CfrSolution`] for your game
/// 3. Select algorithm variants via `S`, `R`, `W`
/// 4. Implement required methods: `batch_size`, `tree_count`, `encoder`, `profile`,
///    `storage`, `advance`
pub trait Solver: Send + Sync {
    /// Turn type classifying nodes as player, chance, or terminal.
    type T: CfrTurn;
    /// Edge type representing actions available at decision points.
    type E: CfrEdge;

    /// Game state handling transitions and payoff evaluation.
    type G: CfrGame<E = Self::E, T = Self::T>;
    /// Information set combining public and private state.
    type I: CfrInfo<E = Self::E, T = Self::T, X = Self::X, Y = Self::Y>;

    /// Public state observable by all players.
    type X: CfrPublic<E = Self::E, T = Self::T>;
    /// Private state observable only by the acting player.
    type Y: CfrSecret;

    /// Solution type storing accumulated regrets and strategy weights.
    type P: CfrSolution<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;
    /// CfrEncoder type mapping game states to information set identifiers.
    type N: CfrEncoder<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;

    /// Strategy weighting scheme for average strategy computation.
    ///
    /// Controls how each iteration's strategy contributes to the final average.
    /// Common choices: [`LinearWeight`] (emphasize recent), [`ConstantWeight`] (uniform).
    type W: WeightSchedule;
    /// Regret update scheme determining CFR variant.
    ///
    /// Controls how regrets are accumulated and discounted over time.
    /// Common choices: [`LinearRegret`] (Pluribus / LCFR), [`DiscountedRegret`] (DCFR), [`FlooredRegret`] (CFR+).
    type R: RegretSchedule;
    /// Sampling strategy for tree traversal.
    ///
    /// Controls which branches are explored during tree construction.
    /// Common choices: [`ExternalSampling`] (sample opponents), [`VanillaSampling`] (vanilla).
    type S: SamplingScheme;

    /// Returns the number of trees to process in each training batch.
    /// Batching allows for more efficient parallel processing of game trees.
    fn batch_size() -> usize;

    /// Returns a reference to the encoder used for converting game states to information sets.
    /// The encoder handles abstraction of game states into trainable buckets.
    fn encoder(&self) -> &Self::N;

    /// Returns a reference to the strategy profile being trained.
    /// The profile tracks accumulated regrets and policies that define the strategy.
    fn profile(&self) -> &Self::P;

    /// Returns a mutable reference to the strategy profile for write access.
    fn storage(&mut self) -> &mut Self::P;

    /// Advances the trainer state to the next iteration
    fn advance(&mut self);

    // automatic implementation

    /// Run one training iteration: batch, update regrets/weight/payoff/count, advance epoch.
    fn step(&mut self) {
        for ref update in self.batch() {
            self.update_regret(update);
            self.update_weight(update);
            self.update_payoff(update);
            self.update_visits(update);
        }
        self.profile().metrics().inspect(|m| m.inc_epoch());
        self.advance();
    }

    /// Runs training for a fixed number of game trees.
    ///
    /// Processes `trees / batch_size` batches, checking for interrupt between each.
    /// For production training, use the trainer binary which calls `step()` directly.
    fn solve(mut self, trees: usize) -> Self
    where
        Self: Sized,
    {
        for _ in 0..trees / Self::batch_size() {
            self.step();
            if rbp_core::interrupted() {
                break;
            }
        }
        self
    }

    /// Run `step()` in a tight loop until `deadline` expires.
    ///
    /// Returns the iteration count and wall-clock elapsed. Used by realtime
    /// / subgame players to burn a fixed wall-clock budget on Monte Carlo
    /// CFR refinement; per-decision regret is harvested at the relevant
    /// infoset via [`Harvest`](super::Harvest).
    fn spend(&mut self, deadline: std::time::Duration) -> (usize, std::time::Duration) {
        let t0 = std::time::Instant::now();
        let iterations = std::iter::repeat_with(|| ())
            .take_while(|()| t0.elapsed() < deadline)
            .map(|()| self.step())
            .count();
        (iterations, t0.elapsed())
    }

    /// Updates accumulated regret values for each edge in the counterfactual.
    ///
    /// Uses the [`RegretSchedule`] associated type (`R`) to determine how regrets
    /// are updated (vanilla, CFR+, discounted, linear).
    fn update_regret(&mut self, cfr: &Decisions<Self::E, Self::I>) {
        let ref info = cfr.info;
        let ref vector = cfr.regret;
        let epoch = self.profile().t();
        for (edge, delta) in vector {
            let total = self.profile().cum_regret(info, edge);
            let updated = Self::R::gain(total, *delta, epoch);
            *self.storage().mut_regret(info, edge) = updated;
        }
    }

    /// Updates accumulated weights for each edge in the counterfactual.
    ///
    /// Uses the [`WeightSchedule`] associated type (`W`) to determine how weights
    /// are accumulated (constant, linear, quadratic, exponential).
    fn update_weight(&mut self, cfr: &Decisions<Self::E, Self::I>) {
        let ref info = cfr.info;
        let ref vector = cfr.policy;
        let epoch = self.profile().t();
        for (edge, delta) in vector {
            let total = self.profile().cum_weight(info, edge);
            let updated = Self::W::learn(total, *delta, epoch);
            *self.storage().mut_weight(info, edge) = updated;
        }
    }

    /// Updates the incremental mean expected value for each edge in the counterfactual.
    ///
    /// Stores the infoset-level EV (V(I)) redundantly for each action as a running
    /// mean. Uses Welford's incremental update: ev += (sample - ev) / (n + 1).
    /// Runs before `update_visits`, so `cum_visits` holds the pre-increment count.
    fn update_payoff(&mut self, cfr: &Decisions<Self::E, Self::I>) {
        let ref info = cfr.info;
        for edge in info.choices() {
            let n = self.profile().cum_visits(info, &edge);
            let ev = self.storage().mut_payoff(info, &edge);
            *ev += (cfr.payoff - *ev) / (n + 1) as rbp_core::Utility;
        }
    }

    /// Updates encounter visits for each edge in the counterfactual.
    ///
    /// Increments the visits for each action in the infoset to track
    /// how many times this info-action pair has been visited during training.
    fn update_visits(&mut self, cfr: &Decisions<Self::E, Self::I>) {
        let ref info = cfr.info;
        for edge in info.choices() {
            *self.storage().mut_visits(info, &edge) += 1;
        }
    }

    /// Product of external (opponent) strategy probabilities along a linear edge path.
    ///
    /// Uses [`CfrEncoder::replay`] to walk **downward** from root, yielding the same
    /// `(T, I, E)` triples as [`Node::decisions`] (which walks **upward**).
    fn external_reach(
        &self,
        root: Self::G,
        hero: Self::T,
        path: impl IntoIterator<Item = Self::E>,
    ) -> rbp_core::Probability {
        self.encoder()
            .replay(root, path)
            .into_iter()
            .filter(|(t, _, _)| t.is_opponent(&hero))
            .map(|(_, ref i, ref e)| self.profile().averaged_policy(i, e))
            .product()
    }

    /// turn a batch of trees into a batch
    /// of infosets into a batch of counterfactual update vectors.
    ///
    /// this encapsulates the largest unit of "update"
    /// that we can generate in parallel / from immutable reference.
    /// it is unclear from RPS benchmarks if:
    /// - what level to parallelize  .collect().into_par_iter()
    /// - what optimal batch size is given N available CPU cores
    /// - for small batches, whether overhead is worth it to parallelize at all
    ///
    /// it would be nice to do a kind of parameter sweep across
    /// these different settings. i should checkout if criterion supports.
    #[cfg(feature = "server")]
    fn batch(&self) -> Vec<Decisions<Self::E, Self::I>> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        // @parallelizable
        (0..Self::batch_size())
            .into_par_iter()
            .map(|i| self.tree(i))
            .map(|t| self.record_tree(t))
            .collect::<Vec<Tree<_, _, _, _>>>()
            .into_par_iter()
            .flat_map(|tree| self.record_infosets(tree))
            .collect::<Vec<InfoSet<_, _, _, _>>>()
            .into_par_iter()
            .map(|infoset| self.update_vector(infoset))
            .collect()
    }
    #[cfg(not(feature = "server"))]
    fn batch(&self) -> Vec<Decisions<Self::E, Self::I>> {
        (0..Self::batch_size())
            .into_iter()
            .map(|i| self.tree(i))
            .map(|tree| self.record_tree(tree))
            .flat_map(|tree| self.record_infosets(tree))
            .map(|infoset| self.update_vector(infoset))
            .collect()
    }

    /// Records tree-level telemetry (`tree_size`, increments node
    /// counter) and returns the tree unchanged so it can be partitioned
    /// downstream.
    fn record_tree(&self, tree: Tree<Self::T, Self::E, Self::G, Self::I>) -> Tree<Self::T, Self::E, Self::G, Self::I> {
        let n = tree.n();
        self.inc_nodes(n);
        #[cfg(feature = "server")]
        rbp_telemetry::metrics::get().mccfr_tree_size.record(n as u64, &[]);
        tree
    }

    /// Partitions a tree by infoset, applies the walker filter, and
    /// records infoset-level telemetry (`infosets_per_tree` per tree,
    /// `infoset_size` per infoset, increments infoset counter).
    fn record_infosets(
        &self,
        tree: Tree<Self::T, Self::E, Self::G, Self::I>,
    ) -> Vec<InfoSet<Self::T, Self::E, Self::G, Self::I>> {
        let walker = self.profile().walker();
        let infosets: Vec<_> = tree
            .partition()
            .into_values()
            .filter(|infoset| infoset.head().game().turn() == walker)
            .collect();
        #[cfg(feature = "server")]
        {
            let tel = rbp_telemetry::metrics::get();
            tel.mccfr_infosets_per_tree.record(infosets.len() as u64, &[]);
            infosets.iter().for_each(|infoset| {
                tel.mccfr_infoset_size.record(infoset.size() as u64, &[]);
                self.inc_infos(1);
            });
        }
        #[cfg(not(feature = "server"))]
        infosets.iter().for_each(|_| self.inc_infos(1));
        infosets
    }

    /// Generate a single tree by growing it DFS from root to leaves.
    ///
    /// `id` is the tree's batch-local identifier (see [`Tree::new`]);
    /// `Solver::batch` passes the par_iter index so trees within a batch
    /// get distinct, deterministic ids.
    fn tree(&self, id: usize) -> Tree<Self::T, Self::E, Self::G, Self::I> {
        TreeBuilder::<_, _, _, _, _, _, Self::S>::new(
            self.encoder(), // embed raw game nodes into abstract Self::Game
            self.profile(), // the current state of the strategy solution
            self.root(),    // root node of the tree
            id,
        )
        .build()
    }

    /// generate the update vectors at a given [InfoSet]. specifically,
    /// calculate the regret and policy for each action, along with
    /// the associated `Info` and expected value.
    /// uses fused regret_and_value to avoid redundant tree traversal.
    fn update_vector(&self, ref infoset: InfoSet<Self::T, Self::E, Self::G, Self::I>) -> Decisions<Self::E, Self::I> {
        let policy = self.profile().policy_vector(infoset);
        let (regret, payoff) = self.profile().dfs(infoset);
        Decisions {
            info: infoset.info(),
            regret,
            policy,
            payoff,
        }
    }

    /// Returns the root node of the game.
    /// This is the starting point for tree generation.
    ///
    /// we currently require that root generation is
    /// from Self::Game, but that could relax to reference &self: Trainer
    fn root(&self) -> Self::G {
        Self::G::root()
    }

    // metrics logging helpers

    fn inc_nodes(&self, n: usize) {
        self.profile().metrics().inspect(|m| m.add_nodes(n));
    }

    fn inc_infos(&self, n: usize) {
        self.profile().metrics().inspect(|m| m.add_infos(n));
    }

    /// Compute exploitability by building full tree and delegating to Profile.
    fn exploitability(&self) -> rbp_core::Utility {
        self.profile().exploitability(
            TreeBuilder::<_, _, _, _, _, _, VanillaSampling>::new(
                self.encoder(),
                self.profile(),
                Self::G::exploitability_root(),
                0,
            )
            .build(),
        )
    }
    /// Monte Carlo exploitability estimate.
    ///
    /// Samples `n` random deals, builds a VanillaSampling tree for each,
    /// computes per-deal exploitability, and averages. Each call to
    /// `exploitability_root()` generates a fresh random deal, so the
    /// average converges to the true expected exploitability at O(1/√n).
    ///
    /// Returns an upper bound on true exploitability because per-deal
    /// best response is less constrained than per-info-set best response
    /// (Jensen's inequality).
    fn mxploitability(&self, n: usize) -> rbp_core::Utility {
        (0..n).map(|_| self.exploitability()).sum::<rbp_core::Utility>() / n as rbp_core::Utility
    }
}
