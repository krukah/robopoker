use crate::*;
use mccfr::*;

/// The Deep CFR loop (Brown et al. 2019, Alg. 1): `K` traversals per epoch fill the memories,
/// then the walker's advantage net refits from scratch; the strategy net fits once, at the end.
/// [`Solver`] folds the same vectors into tables instead. The paper assumes [`ExternalSampling`].
pub trait DeepSolver
where
    Self: Grower<P = DeepProfile<<Self as DeepSolver>::Net>> + Send + Sync,
{
    type Net: RegretNet<F = Self::F> + StrategyNet;

    fn profile_mut(&mut self) -> &mut Self::P;

    fn traversals() -> usize {
        DeepHyperParams::get().traversals()
    }

    /// Run `epochs` single-walker epochs, then fit the average strategy.
    fn solve(mut self, epochs: usize) -> Self
    where
        Self: Sized,
    {
        (0..epochs).for_each(|_| self.step());
        self.profile_mut().net_mut().fit_strategy();
        self
    }

    /// Every traversal reads the nets frozen at this epoch, so observation waits until all finish.
    fn step(&mut self) {
        let walker = self.profile().walker();
        let batch = self.batch();
        self.profile_mut().observe(batch);
        self.profile_mut().net_mut().fit_regret(walker);
        self.profile_mut().increment();
    }

    /// One parallel pass over `K` traversals: regret rows from the walker's infosets, strategy
    /// rows from the infosets its opponents sampled through.
    #[cfg(feature = "server")]
    fn batch(&self) -> TrainingBatch<Self::F> {
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        let walker = self.profile().walker();
        (0..Self::traversals())
            .into_par_iter()
            .flat_map_iter(|id| self.tree(id).partition().into_values())
            .fold(TrainingBatch::new, |mut batch, infoset| {
                batch.capture(self, &infoset, walker);
                batch
            })
            .reduce(TrainingBatch::new, |mut batch, other| {
                batch.append(other);
                batch
            })
    }
    #[cfg(not(feature = "server"))]
    fn batch(&self) -> TrainingBatch<Self::F> {
        let walker = self.profile().walker();
        (0..Self::traversals())
            .flat_map(|id| self.tree(id).partition().into_values())
            .fold(TrainingBatch::new(), |mut batch, infoset| {
                batch.capture(self, &infoset, walker);
                batch
            })
    }

    fn regret(&self, infoset: &InfoSet<Self::F>) -> Sample<Self::F, Regret> {
        Sample::new(infoset.info(), self.profile().iteration(), self.profile().dfs(infoset).0)
    }

    fn policy(&self, infoset: &InfoSet<Self::F>) -> Sample<Self::F, Policy> {
        Sample::new(infoset.info(), self.profile().iteration(), self.profile().iterated_strategy(&infoset.info()))
    }
}
