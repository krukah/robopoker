//! DB-backed lazy blueprint store for RAM-free subgame solving.
//!
//! The subgame solvers ([`subgame::DepthSolver`], [`subgame::WorldSolver`],
//! [`subgame::SubGameSolver`]) touch the blueprint only through [`RefProf`] +
//! [`CfrSampling`], over a *bounded* working set — the infosets reachable
//! inside the subtree, plus their warmstart priors. Nothing forces the whole
//! 347 MB–3 GB blueprint into RAM except that `RefProf` is synchronous, so
//! the loaded [`NlheProfile`] can't `.await` a query inside `mut_weight`.
//!
//! [`LazyBlueprint`] breaks that: it implements the *same* sync `RefProf`,
//! but backs each `(info, edge)` lookup with a memoized, blocking `fetch`
//! closure. The first touch of an info pulls its rows (one query per info,
//! all edges); every later touch is a HashMap hit. A 50k-iteration solve
//! reuses the same finite working set, so the store converges to O(subtree)
//! — kilobytes — and the blueprint stays in Postgres.
//!
//! It plugs into the generic [`Adapt`] sampler as its blueprint type — see
//! [`crate::adapt`]. The crate stays runtime-agnostic: `fetch` is a plain
//! blocking closure. The caller (which owns a tokio runtime + `Client`)
//! bridges it to an async DB actor — see `bin/sparse`.
//!
//! ## Known divergence
//!
//! [`Memory`] defaults a missing edge to its *prior* (`Edge::policy()` /
//! `Edge::regret()`), while in-memory [`NlheProfile`] defaults to `0`. So on
//! *untrained* edges the two disagree; on trained infos (the working set)
//! every edge is present and they match byte-for-byte. This mirrors the
//! existing online `nlhe::lookup` semantics, and is arguably the more
//! correct default — but it is a divergence worth naming.
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
    /// Never-read target for the [`MutProf`] impl. The blueprint is always
    /// consumed behind a shared `&P` (subgame solvers only mutate their own
    /// local `WorldProfile`/`DepthProfile` and read the blueprint via
    /// [`RefProf::warmstart`]), so these `&mut` methods are unreachable at
    /// runtime — the impl exists solely to satisfy the `CfrSolution` bound
    /// that [`subgame::WorldSolver`] places on the blueprint type.
    scratch: Encounter,
}

impl LazyBlueprint {
    /// `epochs` is the blueprint's trained-iteration count (needed by
    /// [`RefProf::warmstart`]); `fetch` blocks until the info's rows arrive.
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
        self.fetches.load(Ordering::Relaxed)
    }

    /// Guarantee `info`'s row-set is memoized. One blocking fetch on miss.
    fn hydrate(&self, info: NlheInfo) {
        if self.memo.lock().expect("memo").contains_key(&info) {
            return;
        }
        let memory = (self.fetch)(info);
        self.fetches.fetch_add(1, Ordering::Relaxed);
        self.memo.lock().expect("memo").entry(info).or_insert(memory);
    }
}

impl CfrRule for LazyBlueprint {
    type T = NlheTurn;
    type E = NlheEdge;
    type G = NlheGame;
    type I = NlheInfo;
}

impl RefProf for LazyBlueprint {
    fn t(&self) -> usize {
        self.epochs
    }

    fn sum_regret(&self) -> Utility {
        0.0
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.hydrate(*info);
        self.memo
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .weight(&Edge::from(*edge))
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.hydrate(*info);
        self.memo
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .regret(&Edge::from(*edge))
    }

    fn cum_payoff(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.hydrate(*info);
        self.memo
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .payoff(&Edge::from(*edge))
    }

    fn cum_visits(&self, info: &Self::I, edge: &Self::E) -> u32 {
        self.hydrate(*info);
        self.memo
            .lock()
            .expect("memo")
            .get(info)
            .expect("hydrated")
            .visits(&Edge::from(*edge))
    }
}

impl CfrSampling for LazyBlueprint {
    fn increment(&mut self) {}

    fn walker(&self) -> Self::T {
        NlheTurn::from(0_usize)
    }
}

impl MutProf for LazyBlueprint {
    fn mut_weight(&mut self, _: &Self::I, _: &Self::E) -> &mut Probability {
        &mut self.scratch.weight
    }

    fn mut_regret(&mut self, _: &Self::I, _: &Self::E) -> &mut Utility {
        &mut self.scratch.regret
    }

    fn mut_payoff(&mut self, _: &Self::I, _: &Self::E) -> &mut Utility {
        &mut self.scratch.payoff
    }

    fn mut_visits(&mut self, _: &Self::I, _: &Self::E) -> &mut u32 {
        &mut self.scratch.visits
    }
}
