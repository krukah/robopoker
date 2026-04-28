use crate::*;

/// One parallel task’s worth of counterfactual updates (internal to [`Solver::batch`]).
struct BatchChunk<E, I>
where
    E: CfrEdge,
    I: CfrInfo<E = E>,
{
    updates: Vec<Counterfactual<E, I>>,
    nodes: usize,
    infos: usize,
}

impl<E, I> Default for BatchChunk<E, I>
where
    E: CfrEdge,
    I: CfrInfo<E = E>,
{
    fn default() -> Self {
        Self {
            updates: Vec::new(),
            nodes: 0,
            infos: 0,
        }
    }
}

/// The core training orchestrator for Monte Carlo CFR.
///
/// Given access to a [`Profile`] and [`Encoder`], the `Solver` trait encapsulates:
/// 1. Sampling game trees via [`SamplingScheme`]
/// 2. Computing [`Counterfactual`] regret vectors at each [`InfoSet`]
/// 3. Updating the [`Profile`] after each batch using [`RegretSchedule`] and [`PolicySchedule`]
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
/// - **`W: PolicySchedule`** — How strategy weights are accumulated across iterations
///   (constant, linear, quadratic, exponential)
///
/// # Implementation
///
/// To implement a solver for a new game:
/// 1. Define game types (Turn, Edge, Game, Info, Public, Secret)
/// 2. Implement [`Encoder`] and [`Profile`] for your game
/// 3. Select algorithm variants via `S`, `R`, `W`
/// 4. Implement required methods: `batch_size`, `tree_count`, `encoder`, `profile`,
///    `advance`, `mut_regret`, `mut_weight`
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

    /// Profile type storing accumulated regrets and strategy weights.
    type P: Profile<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;
    /// Encoder type mapping game states to information set identifiers.
    type N: Encoder<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;

    /// Strategy weighting scheme for average strategy computation.
    ///
    /// Controls how each iteration's strategy contributes to the final average.
    /// Common choices: [`LinearWeight`] (emphasize recent), [`ConstantWeight`] (uniform).
    type W: PolicySchedule;
    /// Regret update scheme determining CFR variant.
    ///
    /// Controls how regrets are accumulated and discounted over time.
    /// Common choices: [`DiscountedRegret`] (DCFR), [`FlooredRegret`] (CFR+).
    type R: RegretSchedule;
    /// Sampling strategy for tree traversal.
    ///
    /// Controls which branches are explored during tree construction.
    /// Common choices: [`ExternalSampling`] (sample opponents), [`VanillaSampling`] (vanilla).
    type S: SamplingScheme;

    /// Returns the number of trees to process in each training batch.
    /// Batching allows for more efficient parallel processing of game trees.
    fn batch_size() -> usize;

    /// Returns the total number of game trees to generate and process during training.
    /// More trees generally leads to better strategy convergence.
    fn tree_count() -> usize;

    /// Returns a reference to the encoder used for converting game states to information sets.
    /// The encoder handles abstraction of game states into trainable buckets.
    fn encoder(&self) -> &Self::N;

    /// Returns a reference to the strategy profile being trained.
    /// The profile tracks accumulated regrets and policies that define the strategy.
    fn profile(&self) -> &Self::P;

    /// Advances the trainer state to the next iteration
    fn advance(&mut self);

    /// Returns a mutable reference to the accumulated regret value for the given infoset/edge pair.
    /// This allows updating the historical regret values that drive strategy updates.
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    /// Returns a mutable reference to the accumulated weight for the given infoset/edge pair.
    /// This allows updating the historical action weights that determine the final strategy.
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    /// Returns a mutable reference to the accumulated expected value for the given infoset/edge pair.
    /// This enables frontier evaluation for depth-limited search and safe subgame solving.
    fn mut_evalue(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32;

    /// Returns a mutable reference to the encounter counts for the given infoset/edge pair.
    /// This tracks how many times each info-action pair has been visited during training.
    fn mut_counts(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32;

    // automatic implementation

    /// Run one training iteration: batch, update regrets/weight/evalue/count, advance epoch.
    fn step(&mut self) {
        for ref update in self.batch() {
            self.update_regret(update);
            self.update_weight(update);
            self.update_evalue(update);
            self.update_counts(update);
        }
        self.profile().metrics().inspect(|m| m.inc_epoch());
        self.advance();
    }

    /// Updates trainer state based on regret vectors from Profile.
    /// NOTE: For production training, use trainer binary which provides unified
    /// interrupt handling and postgres integration.
    fn solve(mut self) -> Self
    where
        Self: Sized,
    {
        for _ in 0..Self::iterations() {
            self.step();
            if rbp_core::interrupted() {
                break;
            }
        }
        self
    }

    /// Updates accumulated regret values for each edge in the counterfactual.
    ///
    /// Uses the [`RegretSchedule`] associated type (`R`) to determine how regrets
    /// are updated (vanilla, CFR+, discounted, linear).
    fn update_regret(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.info;
        let ref vector = cfr.regret;
        let epoch = self.profile().epochs();
        for (edge, delta) in vector.iter() {
            let total = self.profile().cum_regret(info, edge);
            let updated = Self::R::gain(total, *delta, epoch);
            *self.mut_regret(info, edge) = updated;
        }
    }

    /// Updates accumulated weights for each edge in the counterfactual.
    ///
    /// Uses the [`PolicySchedule`] associated type (`W`) to determine how weights
    /// are accumulated (constant, linear, quadratic, exponential).
    fn update_weight(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.info;
        let ref vector = cfr.policy;
        let epoch = self.profile().epochs();
        for (edge, delta) in vector.iter() {
            let total = self.profile().cum_weight(info, edge);
            let updated = Self::W::learn(total, *delta, epoch);
            *self.mut_weight(info, edge) = updated;
        }
    }

    /// Updates accumulated expected values for each edge in the counterfactual.
    ///
    /// Stores the infoset-level EV (V(I)) redundantly for each action.
    /// This denormalization enables quick frontier evaluation without a
    /// separate infoset->EV map. Uses [`PolicySchedule`] (`W`) for weighting.
    fn update_evalue(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.info;
        let ref info_ev = cfr.evalue;
        for ref edge in info.choices() {
            *self.mut_evalue(info, edge) = *info_ev;
        }
    }

    /// Updates encounter counts for each edge in the counterfactual.
    ///
    /// Increments the counts for each action in the infoset to track
    /// how many times this info-action pair has been visited during training.
    fn update_counts(&mut self, cfr: &Counterfactual<Self::E, Self::I>) {
        let ref info = cfr.info;
        for ref edge in info.choices() {
            *self.mut_counts(info, edge) += 1;
        }
    }

    /// Computes external (opponent) reach probability along a linear edge path.
    ///
    /// Iterates through edges, tracking game state. For each opponent decision
    /// point (not hero, not chance), looks up the averaged policy probability.
    /// Returns the product of all opponent action probabilities.
    fn external_reach(
        &self,
        node: Self::G,
        hero: Self::T,
        path: impl IntoIterator<Item = Self::E>,
    ) -> rbp_core::Probability {
        path.into_iter()
            .scan((node, Vec::new()), |(game, past), edge| {
                past.push(edge);
                *game = game.apply(edge);
                match game.turn() {
                    t if t == hero => None,
                    t if t.is_chance() => None,
                    _ => {
                        let info = self.encoder().resume(past, game);
                        Some(self.profile().averaged(&info, &edge))
                    }
                }
            })
            .product()
    }

    /// Build one batch of counterfactual updates.
    ///
    /// With `server`, trees are split into chunks sized from [`Self::batch_chunk_size`]
    /// so each Rayon task does tree grow → partition → counterfactual locally (better
    /// locality than over-parallelizing per-infoset work). NUMA pinning is left to process
    /// / environment (`taskset`, `numactl`, `RAYON_NUM_THREADS`).
    ///
    /// Runs `trees` contiguous trees sequentially (one parallel task's workload).
    fn batch_chunk(&self, trees: usize) -> BatchChunk<Self::E, Self::I> {
        let mut chunk = BatchChunk::default();
        for _ in 0..trees {
            let tree = self.tree();
            chunk.nodes += tree.n();
            let infosets = tree
                .partition()
                .into_values()
                .filter(|infoset| infoset.head().game().turn() == self.profile().walker())
                .collect::<Vec<_>>();
            chunk.infos += infosets.len();
            chunk.updates.extend(
                infosets
                    .into_iter()
                    .map(|infoset| self.counterfactual(infoset)),
            );
        }
        chunk
    }

    /// Parallel chunk size derived from logical CPU count (NUMA-aware placement is external).
    fn batch_chunk_size() -> usize {
        let cores = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        Self::batch_size().div_ceil(cores).max(1)
    }

    #[cfg(feature = "server")]
    fn batch(&self) -> Vec<Counterfactual<Self::E, Self::I>> {
        use rayon::prelude::*;
        let batch_size = Self::batch_size();
        let chunk_size = Self::batch_chunk_size().max(1);
        let num_chunks = batch_size.div_ceil(chunk_size);
        let chunks: Vec<_> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk_idx| {
                let start = chunk_idx * chunk_size;
                let trees = (batch_size - start).min(chunk_size);
                self.batch_chunk(trees)
            })
            .collect();
        let mut nodes = 0usize;
        let mut infos = 0usize;
        let mut updates = Vec::new();
        for chunk in chunks {
            nodes += chunk.nodes;
            infos += chunk.infos;
            updates.extend(chunk.updates);
        }
        self.inc_nodes(nodes);
        self.inc_infos(infos);
        updates
    }

    #[cfg(not(feature = "server"))]
    fn batch(&self) -> Vec<Counterfactual<Self::E, Self::I>> {
        let batch_size = Self::batch_size();
        let chunk = self.batch_chunk(batch_size);
        self.inc_nodes(chunk.nodes);
        self.inc_infos(chunk.infos);
        chunk.updates
    }

    /// Generate a single tree by growing it DFS from root to leaves.
    ///
    /// Delegates to [`TreeBuilder`] for lazy, iterator-based construction.
    fn tree(&self) -> Tree<Self::T, Self::E, Self::G, Self::I> {
        TreeBuilder::<_, _, _, _, _, _, Self::S>::new(
            self.encoder(), // embed raw game nodes into abstract Self::Game
            self.profile(), // the current state of the strategy solution
            self.root(),    // root node of the tree
        )
        .build()
    }

    /// generate the update vectors at a given [InfoSet]. specifically,
    /// calculate the regret and policy for each action, along with
    /// the associated [Info] and expected value
    fn counterfactual(
        &self,
        ref infoset: InfoSet<Self::T, Self::E, Self::G, Self::I>,
    ) -> Counterfactual<Self::E, Self::I> {
        Counterfactual {
            info: infoset.info(),
            regret: self.profile().regret_vector(infoset),
            policy: self.profile().policy_vector(infoset),
            evalue: self.profile().infoset_value(infoset),
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

    /// Returns the number of iterations to run the training loop.
    /// This is calculated as the total number of trees to generate
    /// divided by the batch size.
    fn iterations() -> usize {
        Self::tree_count() / Self::batch_size()
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
                self.root(),
            )
            .build(),
        )
    }
}
