use crate::*;
use pokerkit::*;

/// The tabular profile for any family: one [`Encounter`] per info-edge pair, the epoch count,
/// and training metrics. Implements [`Store`], and so every access trait and all the CFR math.
pub struct Book<F>
where
    F: CfrInfo,
{
    epochs: usize,
    encounters: FxHashMap<F, FxHashMap<EdgeOf<F>, Encounter>>,
    metrics: Metrics,
}

/// Implement this one accessor trait — encounters as `FxHashMap<I, FxHashMap<E, Encounter>>` plus a
/// `usize` epoch — and get blanket [`Storage`], [`Profile`], and [`Clock`]
/// impls.
pub trait Store: CfrSpec + Sized {
    /// Batches with fewer updates fold in place: below this, rayon's dispatch costs more than
    /// the folds it spreads. Measured crossover on Leduc is ~200 updates.
    const SHARDED: usize = 256;

    fn encounters_ref(&self) -> &FxHashMap<InfoOf<Self>, FxHashMap<EdgeOf<Self>, Encounter>>;
    fn encounters_mut(&mut self) -> &mut FxHashMap<InfoOf<Self>, FxHashMap<EdgeOf<Self>, Encounter>>;
    fn epochs_ref(&self) -> usize;
    fn epochs_mut(&mut self) -> &mut usize;
    fn store_metrics(&self) -> Option<&Metrics>;

    /// Folds a batch into the encounters, distinct infosets in parallel once the batch reaches
    /// [`Store::SHARDED`]. Each infoset keeps its batch order, so the result is bit-identical
    /// to [`Solver::apply`]'s serial loop.
    fn absorb<R, W>(&mut self, updates: Batch<Self::F>)
    where
        R: RegretSchedule,
        W: WeightSchedule,
        Self::F: 'static,
    {
        if updates.len() < Self::SHARDED {
            self.inline::<R, W>(updates);
        } else {
            self.shard::<R, W>(updates);
        }
    }

    /// Folds each infoset's run of updates on the driving thread, in batch order.
    fn inline<R, W>(&mut self, updates: Batch<Self::F>)
    where
        R: RegretSchedule,
        W: WeightSchedule,
    {
        let epoch = self.epochs_ref();
        let tables = self
            .lift(&updates.grouped())
            .into_iter()
            .map(|shard| shard.fold::<R, W>(epoch))
            .collect::<Vec<_>>();
        self.encounters_mut().extend(tables);
    }

    /// Lifts each infoset's table out of the map, folds the shards, puts the tables back, and
    /// frees the batch off the driving thread.
    fn shard<R, W>(&mut self, updates: Batch<Self::F>)
    where
        R: RegretSchedule,
        W: WeightSchedule,
        Self::F: 'static,
    {
        let tables = Self::folds::<R, W>(self.lift(&updates.grouped()), self.epochs_ref());
        self.encounters_mut().extend(tables);
        updates.release();
    }

    /// Takes each updated infoset's table out of the map, paired with its run of updates.
    fn lift<'batch>(&mut self, grouped: &'batch [&'batch Decisions<Self::F>]) -> Vec<Shard<'batch, Self::F>> {
        grouped
            .chunk_by(|a, b| a.info() == b.info())
            .map(|run| Shard::new(*run[0].info(), self.encounters_mut().remove(run[0].info()).unwrap_or_default(), run))
            .collect()
    }

    #[cfg(feature = "server")]
    fn folds<R, W>(
        shards: Vec<Shard<'_, Self::F>>,
        epoch: usize,
    ) -> Vec<(InfoOf<Self>, FxHashMap<EdgeOf<Self>, Encounter>)>
    where
        R: RegretSchedule,
        W: WeightSchedule,
    {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        shards.into_par_iter().map(|shard| shard.fold::<R, W>(epoch)).collect()
    }
    #[cfg(not(feature = "server"))]
    fn folds<R, W>(
        shards: Vec<Shard<'_, Self::F>>,
        epoch: usize,
    ) -> Vec<(InfoOf<Self>, FxHashMap<EdgeOf<Self>, Encounter>)>
    where
        R: RegretSchedule,
        W: WeightSchedule,
    {
        shards.into_iter().map(|shard| shard.fold::<R, W>(epoch)).collect()
    }
}

impl<P> Storage for P
where
    P: Store,
{
    fn mut_weight(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Probability {
        self.encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .weight_mut()
    }

    fn mut_regret(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Utility {
        self.encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .regret_mut()
    }

    fn mut_payoff(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Utility {
        self.encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .payoff_mut()
    }

    fn mut_visits(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut u32 {
        self.encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .visits_mut()
    }
}

impl<P> Profile for P
where
    P: Store,
{
    fn t(&self) -> usize {
        self.epochs_ref()
    }

    fn metrics(&self) -> Option<&Metrics> {
        self.store_metrics()
    }

    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(Encounter::weight)
            .unwrap_or_default()
    }

    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map_or_else(|| edge.default_regret(), Encounter::regret)
    }

    fn cum_payoff(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(Encounter::payoff)
            .unwrap_or_default()
    }

    fn cum_visits(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> u32 {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(Encounter::visits)
            .unwrap_or_default()
    }

    fn sum_regret(&self) -> Utility {
        self.encounters_ref()
            .values()
            .flat_map(FxHashMap::values)
            .map(|e| e.regret().max(0.))
            .sum::<Utility>()
            / self.epochs_ref().max(1) as Utility
    }
}

impl<P> Clock for P
where
    P: Store,
{
    fn increment(&mut self) {
        *self.epochs_mut() += 1;
    }
}

impl<F> Default for Book<F>
where
    F: CfrInfo,
{
    fn default() -> Self {
        Self::new(0, FxHashMap::default())
    }
}

impl<F> CfrSpec for Book<F>
where
    F: CfrInfo,
{
    type F = F;
}

impl<F> Store for Book<F>
where
    F: CfrInfo,
{
    fn encounters_ref(&self) -> &FxHashMap<F, FxHashMap<EdgeOf<F>, Encounter>> {
        self.encounters()
    }

    fn encounters_mut(&mut self) -> &mut FxHashMap<F, FxHashMap<EdgeOf<F>, Encounter>> {
        &mut self.encounters
    }

    fn epochs_ref(&self) -> usize {
        self.epochs()
    }

    fn epochs_mut(&mut self) -> &mut usize {
        &mut self.epochs
    }

    fn store_metrics(&self) -> Option<&Metrics> {
        Some(self.metrics())
    }
}

impl<F> Book<F>
where
    F: CfrInfo,
{
    /// A profile restored at `epochs`, with metrics resuming from there.
    pub fn new(epochs: usize, encounters: FxHashMap<F, FxHashMap<EdgeOf<F>, Encounter>>) -> Self {
        Self {
            epochs,
            encounters,
            metrics: Metrics::with_epoch(epochs),
        }
    }

    pub fn epochs(&self) -> usize {
        self.epochs
    }

    pub fn encounters(&self) -> &FxHashMap<F, FxHashMap<EdgeOf<F>, Encounter>> {
        &self.encounters
    }

    fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}
