use crate::*;

/// The core training orchestrator for Monte Carlo CFR.
///
/// Bundles the game types (`T`/`E`/`G`/`I`/`X`/`Y`) with the three algorithm
/// variants — `S` sampling, `R` regret, `W` weight — that pin down the CFR
/// flavor, then drives sample → regret → update batches over a [`CfrSolution`].
pub trait Solver: Send + Sync {
    type T: CfrTurn;
    type E: CfrEdge;

    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T, X = Self::X, Y = Self::Y>;

    type X: CfrPublic<E = Self::E, T = Self::T>;
    type Y: CfrSecret;

    type P: CfrSolution<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;
    type N: CfrEncoder<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;

    /// Weighting of each iteration's contribution to the average strategy.
    type W: WeightSchedule;
    /// CFR variant: [`LinearRegret`] is LCFR / Pluribus parity, [`DiscountedRegret`] is DCFR, [`FlooredRegret`] is CFR+.
    type R: RegretSchedule;
    /// Which branches get explored during tree construction.
    type S: SamplingScheme;

    /// Number of trees per training batch.
    fn batch_size() -> usize;

    fn encoder(&self) -> &Self::N;

    fn profile(&self) -> &Self::P;

    fn storage(&mut self) -> &mut Self::P;

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

    /// Train over `trees / batch_size` batches, checking for interrupt between each.
    ///
    /// For production training, use the trainer binary which calls `step()` directly.
    fn solve(mut self, trees: usize) -> Self
    where
        Self: Sized,
    {
        for _ in 0..trees / Self::batch_size() {
            self.step();
            if pokerkit::interrupted() {
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

    /// Accumulate regret for each edge in the counterfactual, discounted per `Self::R`.
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

    /// Accumulate strategy weight for each edge in the counterfactual, per `Self::W`.
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

    /// Running mean of the infoset-level EV V(I), stored redundantly per action.
    ///
    /// Welford's incremental update: `ev += (sample - ev) / (n + 1)`. Must run
    /// before `update_visits`, so `cum_visits` holds the pre-increment count.
    fn update_payoff(&mut self, cfr: &Decisions<Self::E, Self::I>) {
        let ref info = cfr.info;
        for edge in info.choices() {
            let n = self.profile().cum_visits(info, &edge);
            let ev = self.storage().mut_payoff(info, &edge);
            *ev += (cfr.payoff - *ev) / (n + 1) as pokerkit::Utility;
        }
    }

    /// Increment the visit count of every info-action pair in the infoset.
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
    ) -> pokerkit::Probability {
        self.encoder()
            .replay(root, path)
            .into_iter()
            .filter(|(t, _, _)| t.is_opponent(&hero))
            .map(|(_, ref i, ref e)| self.profile().averaged_policy(i, e))
            .product()
    }

    /// Folds this worker thread's CPU time for `f` into the batch CPU meter
    /// (see the `cpu` module), so telemetry reports utilization, not wall-clock.
    ///
    /// Applied only at *per-tree* granularity (tree build + partition), never
    /// per-infoset: `clock_gettime` is a real syscall and a batch has ~1e5
    /// infosets, so per-infoset metering would add ~1e5 syscalls plus atomic
    /// contention on the shared counter — degrading the very parallelism it
    /// measures. Tree build + partition dominate CPU, so utilization stays
    /// representative (it mildly undercounts by omitting regret matching).
    fn measured<R>(&self, f: impl FnOnce() -> R) -> R {
        let (out, cpu) = crate::cpu::measure(f);
        self.profile().metrics().inspect(|m| m.add_cpu(cpu));
        out
    }

    /// Trees → infosets → counterfactual update vectors: the largest unit of
    /// "update" derivable in parallel from an immutable reference.
    ///
    /// RPS benchmarks left open which level to parallelize, what batch size is
    /// optimal for N cores, and whether small batches are worth the overhead at
    /// all. A parameter sweep (criterion?) would settle it.
    #[cfg(feature = "server")]
    fn batch(&self) -> Vec<Decisions<Self::E, Self::I>> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        // @parallelizable — per-tree work is CPU-metered; the per-infoset update
        // stage is intentionally left unmetered (see `measured`).
        (0..Self::batch_size())
            .into_par_iter()
            .map(|i| self.measured(|| self.tree(i)))
            .map(|t| self.record_tree(t))
            .collect::<Vec<Tree<_, _, _, _>>>()
            .into_par_iter()
            .flat_map(|tree| self.measured(|| self.record_infosets(tree)))
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

    /// Records tree-level telemetry and returns the tree unchanged.
    fn record_tree(&self, tree: Tree<Self::T, Self::E, Self::G, Self::I>) -> Tree<Self::T, Self::E, Self::G, Self::I> {
        let n = tree.n();
        self.inc_nodes(n);
        #[cfg(feature = "server")]
        vitals::metrics::get().mccfr_tree_size.record(n as u64, &[]);
        tree
    }

    /// Partitions a tree by infoset, keeps only the walker's, records telemetry.
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
        infosets.iter().for_each(|_| self.inc_infos(1));
        #[cfg(feature = "server")]
        {
            let tel = vitals::metrics::get();
            tel.mccfr_infosets_per_tree.record(infosets.len() as u64, &[]);
            infosets
                .iter()
                .for_each(|infoset| tel.mccfr_infoset_size.record(infoset.size() as u64, &[]));
        }
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

    /// Regret + policy + EV vectors at a given [InfoSet].
    ///
    /// The fused `dfs` returns regret and value together, avoiding a second traversal.
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

    /// Root node for tree generation.
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

    /// Exploitability over a fully-expanded tree, delegated to the Profile.
    fn exploitability(&self) -> pokerkit::Utility {
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
    /// Monte Carlo exploitability estimate over `n` random deals.
    ///
    /// `exploitability_root()` deals fresh each call, so the mean converges to
    /// true expected exploitability at O(1/√n). It is an *upper* bound: per-deal
    /// best response is less constrained than per-infoset (Jensen's inequality).
    fn mxploitability(&self, n: usize) -> pokerkit::Utility {
        (0..n).map(|_| self.exploitability()).sum::<pokerkit::Utility>() / n as pokerkit::Utility
    }
}
