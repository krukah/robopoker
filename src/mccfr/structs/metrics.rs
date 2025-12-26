use super::TrainingStats;
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Instant;

thread_local! { static LOCAL_EPOCH: Cell<usize> = const { Cell::new(0) }; }

/// Thread-local accumulated metrics for CFR training.
/// Uses thread-local counter for epochs to avoid contention in parallel execution.
/// Nodes and infos use direct atomic adds since they're counted at batch boundaries.
/// Owns timing for both started and periodic checkpoint logging.
pub struct Metrics {
    epoch: AtomicUsize,
    nodes: AtomicUsize,
    infos: AtomicUsize,
    start: Instant,
    check: Mutex<Instant>,
}

impl Default for Metrics {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            epoch: AtomicUsize::new(0),
            nodes: AtomicUsize::new(0),
            infos: AtomicUsize::new(0),
            start: now,
            check: Mutex::new(now),
        }
    }
}

impl Metrics {
    pub fn inc_epoch(&self) {
        LOCAL_EPOCH.with(|c| c.set(c.get() + 1));
    }
    pub fn add_nodes(&self, n: usize) {
        self.nodes.fetch_add(n, Ordering::Relaxed);
    }
    pub fn add_infos(&self, n: usize) {
        self.infos.fetch_add(n, Ordering::Relaxed);
    }
    pub fn flush(&self) {
        LOCAL_EPOCH.with(|c| self.epoch.fetch_add(c.replace(0), Ordering::Relaxed));
    }
    /// Returns stats only if checkpoint interval has elapsed.
    /// Updates checkpoint time when stats are returned.
    pub fn checkpoint(&self) -> Option<String> {
        let mut last = self.check.lock().expect("poision");
        if last.elapsed() >= crate::TRAINING_LOG_INTERVAL {
            *last = Instant::now();
            std::mem::drop(last);
            Some(self.stats())
        } else {
            None
        }
    }
}

impl TrainingStats for Metrics {
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
