use crate::*;
use mccfr::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Wraps a net and counts its forward passes, so tests can see what the memos save.
pub struct Counted<NN>
where
    NN: RegretNet + StrategyNet,
{
    inner: NN,
    regrets: AtomicUsize,
    weights: AtomicUsize,
}

impl<NN> CfrSpec for Counted<NN>
where
    NN: RegretNet + StrategyNet,
{
    type F = NN::F;
}

impl<NN> RegretNet for Counted<NN>
where
    NN: RegretNet + StrategyNet,
{
    fn est_regret(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Regret> {
        self.regrets_ref().fetch_add(1, Ordering::Relaxed);
        self.inner().est_regret(info)
    }

    fn see_regret(&mut self, walker: TurnOf<Self>, sample: Sample<Self::F, Regret>) {
        self.inner_mut().see_regret(walker, sample);
    }

    fn fit_regret(&mut self, walker: TurnOf<Self>) {
        self.inner_mut().fit_regret(walker);
    }
}

impl<NN> StrategyNet for Counted<NN>
where
    NN: RegretNet + StrategyNet,
{
    fn est_strategy(&self, info: &InfoOf<Self>) -> Over<EdgeOf<Self>, Policy> {
        self.weights_ref().fetch_add(1, Ordering::Relaxed);
        self.inner().est_strategy(info)
    }

    fn see_strategy(&mut self, sample: Sample<Self::F, Policy>) {
        self.inner_mut().see_strategy(sample);
    }

    fn fit_strategy(&mut self) {
        self.inner_mut().fit_strategy();
    }
}

impl<NN> Counted<NN>
where
    NN: RegretNet + StrategyNet,
{
    pub fn new(inner: NN) -> Self {
        Self {
            inner,
            regrets: AtomicUsize::new(0),
            weights: AtomicUsize::new(0),
        }
    }

    /// Calls to `est_regret` so far.
    pub fn regrets(&self) -> usize {
        self.regrets_ref().load(Ordering::Relaxed)
    }

    /// Calls to `est_strategy` so far.
    pub fn weights(&self) -> usize {
        self.weights_ref().load(Ordering::Relaxed)
    }

    fn inner(&self) -> &NN {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut NN {
        &mut self.inner
    }

    fn regrets_ref(&self) -> &AtomicUsize {
        &self.regrets
    }

    fn weights_ref(&self) -> &AtomicUsize {
        &self.weights
    }
}
