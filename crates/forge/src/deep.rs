//! Deep CFR training on NLHE: no database, no abstraction tables.
use mccfr::*;
use neural::*;
use nlhe::FullSolver;
use std::path::PathBuf;
use std::time::Instant;

/// Deep CFR over [`nlhe::FullInfo`], epoch after epoch until interrupted. Every flush interval,
/// and once at the end, it fits the average strategy and saves that net into `dir`.
pub struct DeepSession {
    solver: FullSolver,
    dir: PathBuf,
    saved: Instant,
    started: Instant,
}

impl DeepSession {
    /// The NLHE preset. `DeepHyperParams::default` stays at the Kuhn values the Nash tests pin.
    fn hyperparams() -> DeepHyperParams {
        DeepHyperParams::new(1 << 10, 1 << 22, 256, 3, 1 << 11, 1 << 11, 1e-3)
    }

    pub fn new(dir: PathBuf) -> Self {
        if let Err(preset) = Self::hyperparams().init() {
            tracing::warn!(?preset, "DeepHyperParams already set; the preset is ignored");
        }
        Self {
            solver: FullSolver::default(),
            dir,
            saved: Instant::now(),
            started: Instant::now(),
        }
    }

    pub fn train(mut self) {
        tracing::info!(hyperparams = ?DeepHyperParams::get(), dir = ?self.dir(), "training deep cfr on nlhe");
        while !pokerkit::Signals::interrupted() {
            self.step();
            if self.saved().elapsed() >= TrainingHyperParams::get().flush_interval() {
                self.save();
            }
        }
        self.save();
        tracing::info!(epochs = self.solver().profile().epoch(), elapsed = ?self.started().elapsed(), "deep cfr stopped");
    }

    fn step(&mut self) {
        let start = Instant::now();
        self.solver_mut().step();
        tracing::info!(epoch = self.solver().profile().epoch(), took = ?start.elapsed(), "deep cfr epoch");
    }

    /// Refits the strategy net from the current memory; later epochs keep adding to that memory.
    fn save(&mut self) {
        let start = Instant::now();
        self.solver_mut().profile_mut().net_mut().fit_strategy();
        self.solver()
            .profile()
            .net()
            .save(self.dir())
            .expect("save strategy net");
        self.saved = Instant::now();
        tracing::info!(epoch = self.solver().profile().epoch(), took = ?start.elapsed(), dir = ?self.dir(), "strategy net saved");
    }

    fn solver(&self) -> &FullSolver {
        &self.solver
    }

    fn solver_mut(&mut self) -> &mut FullSolver {
        &mut self.solver
    }

    fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    fn saved(&self) -> Instant {
        self.saved
    }

    fn started(&self) -> Instant {
        self.started
    }
}
