//! DB-backed lazy blueprint store for RAM-free subgame solving.
//!
//! The subgame solvers ([`subgame::DepthSolver`], [`subgame::WorldSolver`],
//! [`subgame::SubGameSolver`]) touch the blueprint only through [`Profile`] + [`Clock`],
//! over a *bounded* working set — the infosets reachable inside the subtree, plus their
//! warmstart priors.
use super::*;
use kicker::*;
use mccfr::*;
use pokerkit::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// A read-only blueprint that never resides in RAM: each info is fetched on
/// first touch through a blocking `fetch` closure and memoized thereafter.
pub struct LazyBlueprint {
    epochs: usize,
    fetch: Box<dyn Fn(NlheInfo) -> Memory + Send + Sync>,
    memo: Mutex<HashMap<NlheInfo, Memory>>,
    fetches: AtomicUsize,
    /// Never-read target for the [`Storage`] impl.
    scratch: Encounter,
}

impl LazyBlueprint {
    /// `epochs` is the blueprint's trained-iteration count (needed by
    /// [`Profile::warmstart`]); `fetch` blocks until the info's rows arrive.
    pub fn new(epochs: usize, fetch: impl Fn(NlheInfo) -> Memory + Send + Sync + 'static) -> Self {
        Self {
            epochs,
            fetch: Box::new(fetch),
            memo: Mutex::new(HashMap::new()),
            fetches: AtomicUsize::new(0),
            scratch: Encounter::default(),
        }
    }

    /// Distinct infos pulled from the store — the realized working-set size.
    pub fn fetches(&self) -> usize {
        self.fetches_ref().load(Ordering::Relaxed)
    }

    /// Guarantee `info`'s row-set is memoized. One blocking fetch on miss.
    fn hydrate(&self, info: NlheInfo) {
        if self.memo().lock().expect("memo").contains_key(&info) {
            return;
        }
        let memory = (self.fetch())(info);
        self.fetches_ref().fetch_add(1, Ordering::Relaxed);
        self.memo().lock().expect("memo").entry(info).or_insert(memory);
    }

    fn fetch(&self) -> &(dyn Fn(NlheInfo) -> Memory + Send + Sync) {
        self.fetch.as_ref()
    }

    fn memo(&self) -> &Mutex<HashMap<NlheInfo, Memory>> {
        &self.memo
    }

    fn fetches_ref(&self) -> &AtomicUsize {
        &self.fetches
    }

    fn scratch_mut(&mut self) -> &mut Encounter {
        &mut self.scratch
    }
}

impl CfrSpec for LazyBlueprint {
    type F = NlheInfo;
}

impl Profile for LazyBlueprint {
    fn t(&self) -> usize {
        self.epochs
    }

    fn sum_regret(&self) -> Utility {
        0.0
    }

    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.hydrate(*info);
        self.memo()
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .weight(&Edge::from(*edge))
    }

    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.hydrate(*info);
        self.memo()
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .regret(&Edge::from(*edge))
    }

    fn cum_payoff(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.hydrate(*info);
        self.memo()
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .payoff(&Edge::from(*edge))
    }

    fn cum_visits(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> u32 {
        self.hydrate(*info);
        self.memo()
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .visits(&Edge::from(*edge))
    }
}

impl Clock for LazyBlueprint {
    fn increment(&mut self) {}

    fn walker(&self) -> TurnOf<Self> {
        NlheTurn::from(0_usize)
    }
}

impl Storage for LazyBlueprint {
    fn mut_weight(&mut self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> &mut Probability {
        self.scratch_mut().weight_mut()
    }

    fn mut_regret(&mut self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> &mut Utility {
        self.scratch_mut().regret_mut()
    }

    fn mut_payoff(&mut self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> &mut Utility {
        self.scratch_mut().payoff_mut()
    }

    fn mut_visits(&mut self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> &mut u32 {
        self.scratch_mut().visits_mut()
    }
}
