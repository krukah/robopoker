use crate::*;
use pokerkit::*;

/// One infoset's share of a batch: its run of updates in batch order, and its encounter table
/// lifted out of the profile so that distinct infosets fold in parallel.
pub struct Shard<'batch, F>
where
    F: CfrInfo,
{
    info: F,
    table: FxHashMap<EdgeOf<F>, Encounter>,
    updates: &'batch [&'batch Decisions<F>],
}

impl<'batch, F> Shard<'batch, F>
where
    F: CfrInfo,
{
    pub fn new(info: F, table: FxHashMap<EdgeOf<F>, Encounter>, updates: &'batch [&'batch Decisions<F>]) -> Self {
        Self { info, table, updates }
    }

    /// Folds every update into the table, in batch order, and hands the table back.
    pub fn fold<R, W>(mut self, epoch: usize) -> (F, FxHashMap<EdgeOf<F>, Encounter>)
    where
        R: RegretSchedule,
        W: WeightSchedule,
    {
        self.updates()
            .iter()
            .for_each(|update| self.accrue::<R, W>(update, epoch));
        (self.info(), self.into_table())
    }

    fn info(&self) -> F {
        self.info
    }

    fn table_mut(&mut self) -> &mut FxHashMap<EdgeOf<F>, Encounter> {
        &mut self.table
    }

    fn into_table(self) -> FxHashMap<EdgeOf<F>, Encounter> {
        self.table
    }

    fn updates(&self) -> &'batch [&'batch Decisions<F>] {
        self.updates
    }

    /// One update folded into the table: the same reads and writes, in the same order, as
    /// `Solver`'s `update_regret`, `update_weight`, `update_payoff`, `update_visits`.
    fn accrue<R, W>(&mut self, update: &Decisions<F>, epoch: usize)
    where
        R: RegretSchedule,
        W: WeightSchedule,
    {
        self.regret::<R>(update, epoch);
        self.weight::<W>(update, epoch);
        self.payoff(update);
        self.visits(update);
    }

    fn regret<R>(&mut self, update: &Decisions<F>, epoch: usize)
    where
        R: RegretSchedule,
    {
        update.regret().into_iter().for_each(|(edge, delta)| {
            let total = self
                .table()
                .get(edge)
                .map_or_else(|| edge.default_regret(), Encounter::regret);
            *self.cell(edge).regret_mut() = R::gain(total, *delta, epoch);
        });
    }

    fn weight<W>(&mut self, update: &Decisions<F>, epoch: usize)
    where
        W: WeightSchedule,
    {
        update.policy().into_iter().for_each(|(edge, delta)| {
            let total = self.table().get(edge).map(Encounter::weight).unwrap_or_default();
            *self.cell(edge).weight_mut() = W::learn(total, *delta, epoch);
        });
    }

    fn payoff(&mut self, update: &Decisions<F>) {
        update.info().menu().for_each(|edge| {
            let n = self.table().get(&edge).map(Encounter::visits).unwrap_or_default();
            let ev = self.cell(&edge).payoff_mut();
            *ev += (update.payoff() - *ev) / (n + 1) as Utility;
        });
    }

    fn visits(&mut self, update: &Decisions<F>) {
        update
            .info()
            .menu()
            .for_each(|edge| *self.cell(&edge).visits_mut() += 1);
    }

    fn cell(&mut self, edge: &EdgeOf<F>) -> &mut Encounter {
        self.table_mut().entry(*edge).or_insert_with(|| Encounter::from(edge))
    }

    fn table(&self) -> &FxHashMap<EdgeOf<F>, Encounter> {
        &self.table
    }
}
