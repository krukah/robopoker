//! Training diagnostics and progress tracking.

mod checkpoint;
mod progress;

pub use checkpoint::*;
pub use progress::*;

use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

/// Worker threads available to the parallel batch (1 without the `server` feature).
#[cfg(feature = "server")]
fn workers() -> usize {
    rayon::current_num_threads().max(1)
}
#[cfg(not(feature = "server"))]
fn workers() -> usize {
    1
}

thread_local! { static LOCAL_EPOCH: Cell<usize> = const { Cell::new(0) }; }

/// Accumulated CFR training counters, plus the timing for periodic checkpoints.
///
/// Epochs go through a thread-local counter to dodge contention; nodes and infos
/// use direct atomic adds since they're only touched at batch boundaries.
pub struct Metrics {
    epoch: AtomicUsize,
    nodes: AtomicUsize,
    infos: AtomicUsize,
    cpu: AtomicU64,
    start: Instant,
    prior: Mutex<(Instant, usize)>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::with_epoch(0)
    }
}

impl Metrics {
    pub fn with_epoch(epoch: usize) -> Self {
        let now = Instant::now();
        Self {
            epoch: AtomicUsize::new(epoch),
            nodes: AtomicUsize::new(0),
            infos: AtomicUsize::new(0),
            cpu: AtomicU64::new(0),
            start: now,
            prior: Mutex::new((now, 0)),
        }
    }
    /// Increments the thread-local epoch counter; call once per iteration.
    pub fn inc_epoch(&self) {
        LOCAL_EPOCH.with(|c| c.set(c.get() + 1));
    }
    /// Atomically adds to the global node count.
    pub fn add_nodes(&self, n: usize) {
        self.nodes.fetch_add(n, Ordering::Relaxed);
    }
    /// Atomically adds to the global info set count.
    pub fn add_infos(&self, n: usize) {
        self.infos.fetch_add(n, Ordering::Relaxed);
    }
    /// Atomically folds a worker thread's consumed CPU time into the batch total.
    /// Summed across Rayon workers, `cpu() / (elapsed * workers)` is CPU utilization.
    pub fn add_cpu(&self, cpu: Duration) {
        self.cpu.fetch_add(cpu.as_nanos() as u64, Ordering::Relaxed);
    }
    /// Total CPU time consumed across all worker threads.
    pub fn cpu(&self) -> Duration {
        Duration::from_nanos(self.cpu.load(Ordering::Relaxed))
    }
    /// Flushes the thread-local epoch count into the shared atomic; must precede
    /// any read of `epoch`.
    pub fn flush(&self) {
        LOCAL_EPOCH.with(|c| self.epoch.fetch_add(c.replace(0), Ordering::Relaxed));
    }
    /// Stats, but only once the checkpoint interval has elapsed.
    ///
    /// The reported rate is per-interval (I/sec since the last checkpoint),
    /// not cumulative.
    pub fn checkpoint(&self) -> Option<Checkpoint> {
        let mut prior = self.prior.lock().expect("poison");
        if prior.0.elapsed() >= crate::TrainingHyperParams::get().log_interval() {
            self.flush();
            let secs = prior.0.elapsed().as_secs().max(1) as f64;
            let curr = self.infos();
            let rate = (curr - prior.1) as f64 / secs;
            *prior = (Instant::now(), curr);
            let util =
                100.0 * self.cpu().as_secs_f64() / (self.start.elapsed().as_secs_f64().max(1.0) * workers() as f64);
            Some(Checkpoint::new(self.epoch(), self.nodes(), curr, rate, util))
        } else {
            None
        }
    }
}

impl Progress for Metrics {
    fn epoch(&self) -> usize {
        self.epoch.load(Ordering::Relaxed)
    }

    fn nodes(&self) -> usize {
        self.nodes.load(Ordering::Relaxed)
    }

    fn infos(&self) -> usize {
        self.infos.load(Ordering::Relaxed)
    }

    fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    fn stats(&self) -> String {
        self.flush();
        self.format()
    }
}
