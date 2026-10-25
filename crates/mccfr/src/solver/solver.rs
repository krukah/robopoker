use crate::*;

/// The core training orchestrator for Monte Carlo CFR.
///
/// Bundles an info type (via [`CfrSpec`]) with the three algorithm
/// variants — `S` sampling, `R` regret, `W` weight — that pin down the CFR
/// flavor, then drives sample → regret → update batches over a [`Solution`].
pub trait Solver
where
    Self: Grower<P: Solution> + Send + Sync,
{
    /// Weighting of each iteration's contribution to the average strategy.
    type W: WeightSchedule;
    /// CFR variant: [`LinearRegret`] is LCFR / Pluribus parity, [`DiscountedRegret`] is DCFR, [`FlooredRegret`] is CFR+.
    type R: RegretSchedule;
    /// One batch in this many records the per-tree histograms and meters tree CPU time. Doing
    /// either on every batch costs workers locks and syscalls. Odd, so both walkers get sampled.
    const SAMPLE_EVERY: usize = 17;

    /// Number of trees per training batch.
    fn batch_size() -> usize;

    fn storage(&mut self) -> &mut Self::P;

    fn advance(&mut self);

    /// Run one training iteration: batch, update regrets/weight/payoff/count, advance epoch.
    fn step(&mut self) {
        self.apply(self.batch());
        self.profile().metrics().into_iter().for_each(Metrics::inc_epoch);
        self.advance();
    }

    /// Fold a batch of updates into the profile, one at a time. Map-backed profiles override
    /// this with [`Store::absorb`], which folds distinct infosets in parallel.
    fn apply(&mut self, updates: Batch<Self::F>) {
        for ref update in updates {
            self.update_regret(update);
            self.update_weight(update);
            self.update_payoff(update);
            self.update_visits(update);
        }
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
            if pokerkit::Signals::interrupted() {
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
    fn update_regret(&mut self, cfr: &Decisions<Self::F>) {
        let info = cfr.info();
        let vector = cfr.regret();
        let epoch = self.profile().t();
        for (edge, delta) in vector {
            let total = self.profile().cum_regret(info, edge);
            let updated = Self::R::gain(total, *delta, epoch);
            *self.storage().mut_regret(info, edge) = updated;
        }
    }

    /// Accumulate strategy weight for each edge in the counterfactual, per `Self::W`.
    fn update_weight(&mut self, cfr: &Decisions<Self::F>) {
        let info = cfr.info();
        let vector = cfr.policy();
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
    fn update_payoff(&mut self, cfr: &Decisions<Self::F>) {
        let info = cfr.info();
        for edge in info.menu() {
            let n = self.profile().cum_visits(info, &edge);
            let ev = self.storage().mut_payoff(info, &edge);
            *ev += (cfr.payoff() - *ev) / (n + 1) as pokerkit::Utility;
        }
    }

    /// Increment the visit count of every info-action pair in the infoset.
    fn update_visits(&mut self, cfr: &Decisions<Self::F>) {
        let info = cfr.info();
        for edge in info.menu() {
            *self.storage().mut_visits(info, &edge) += 1;
        }
    }

    /// Product of external (opponent) strategy probabilities along a linear edge path.
    ///
    /// Uses [`Encoding::replay`] to walk **downward** from root, yielding the same
    /// [`Choice`]s as [`Node::decisions`] (which walks **upward**).
    fn external_reach(
        &self,
        root: GameOf<Self>,
        hero: TurnOf<Self>,
        path: impl IntoIterator<Item = EdgeOf<Self>>,
    ) -> pokerkit::Probability {
        self.encoder()
            .replay(root, path)
            .into_iter()
            .filter(|choice| choice.turn().is_opponent(&hero))
            .map(|choice| self.profile().averaged_policy(choice.info(), choice.edge()))
            .product()
    }

    /// On sampled batches, folds this worker thread's CPU time for `f` into the CPU meter (see the
    /// `cpu` module), scaled by [`Solver::SAMPLE_EVERY`] to stand for the batches it skips.
    fn measured<R>(&self, f: impl FnOnce() -> R) -> R {
        if self.sampled() {
            let (out, cpu) = crate::cpu::Cpu::measure(f);
            self.profile()
                .metrics()
                .inspect(|m| m.add_cpu(cpu * Self::SAMPLE_EVERY as u32));
            out
        } else {
            f()
        }
    }

    /// Trees → infosets → counterfactual update vectors: the largest unit of
    /// "update" derivable in parallel from an immutable reference.
    ///
    /// RPS benchmarks left open which level to parallelize, what batch size is
    /// optimal for N cores, and whether small batches are worth the overhead at
    /// all. A parameter sweep (criterion?) would settle it.
    #[cfg(feature = "server")]
    fn batch(&self) -> Batch<Self::F> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        // @parallelizable — per-tree work is CPU-metered; the per-infoset update
        // stage is intentionally left unmetered (see `measured`).
        (0..Self::batch_size())
            .into_par_iter()
            .map(|i| self.measured(|| self.tree(i)))
            .map(|t| self.record_tree(t))
            .map(|tree| self.measured(|| self.record_infosets(tree)))
            .map(|infosets| {
                infosets
                    .into_par_iter()
                    .map(|infoset| self.update_vector(infoset))
                    .collect()
            })
            .collect::<Vec<_>>()
            .into()
    }
    #[cfg(not(feature = "server"))]
    fn batch(&self) -> Batch<Self::F> {
        (0..Self::batch_size())
            .into_iter()
            .map(|i| self.tree(i))
            .map(|tree| self.record_tree(tree))
            .map(|tree| self.record_infosets(tree))
            .map(|infosets| {
                infosets
                    .into_iter()
                    .map(|infoset| self.update_vector(infoset))
                    .collect()
            })
            .collect::<Vec<_>>()
            .into()
    }

    /// Records tree-level telemetry and returns the tree unchanged.
    fn record_tree(&self, tree: Tree<Self::F>) -> Tree<Self::F> {
        self.inc_nodes(tree.n());
        #[cfg(feature = "server")]
        if self.sampled() {
            vitals::metrics::Handles::get()
                .mccfr_tree_size()
                .record(tree.n() as u64, &[]);
        }
        tree
    }

    /// Partitions a tree by infoset, keeps only the walker's, records telemetry.
    fn record_infosets(&self, tree: Tree<Self::F>) -> Vec<InfoSet<Self::F>> {
        let walker = self.profile().walker();
        let infosets = tree
            .partition()
            .into_values()
            .filter(|infoset| infoset.head().game().turn() == walker)
            .collect::<Vec<_>>();
        self.inc_infos(infosets.len());
        #[cfg(feature = "server")]
        if self.sampled() {
            let tel = vitals::metrics::Handles::get();
            tel.mccfr_infosets_per_tree().record(infosets.len() as u64, &[]);
            infosets
                .iter()
                .for_each(|infoset| tel.mccfr_infoset_size().record(infoset.size() as u64, &[]));
        }
        infosets
    }

    /// Regret + policy + EV vectors at a given [InfoSet].
    ///
    /// The fused `dfs` returns regret and value together, avoiding a second traversal.
    fn update_vector(&self, ref infoset: InfoSet<Self::F>) -> Decisions<Self::F> {
        let policy = self.profile().policy_vector(infoset);
        let (regret, payoff) = self.profile().dfs(infoset);
        Decisions::new(infoset.info(), regret, policy, payoff)
    }

    /// Whether this batch records the per-tree and per-infoset histograms.
    fn sampled(&self) -> bool {
        self.profile().t().is_multiple_of(Self::SAMPLE_EVERY)
    }

    fn inc_nodes(&self, n: usize) {
        self.profile().metrics().inspect(|m| m.add_nodes(n));
    }

    fn inc_infos(&self, n: usize) {
        self.profile().metrics().inspect(|m| m.add_infos(n));
    }
}
