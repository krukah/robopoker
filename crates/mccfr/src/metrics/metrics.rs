use crate::Progress;
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
            start: now,
            prior: Mutex::new((now, 0)),
        }
    }
    /// Increments the thread-local epoch counter.
    /// Call once per training iteration.
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
    /// Flushes thread-local epoch count to the shared atomic.
    /// Call before reading epoch to ensure accuracy.
    pub fn flush(&self) {
        LOCAL_EPOCH.with(|c| self.epoch.fetch_add(c.replace(0), Ordering::Relaxed));
    }
    /// Returns stats only if checkpoint interval has elapsed.
    /// Updates checkpoint time when stats are returned.
    /// Reports interval rate (I/sec since last checkpoint) rather than cumulative.
    pub fn checkpoint(&self) -> Option<String> {
        let mut prior = self.prior.lock().expect("poison");
        if prior.0.elapsed() >= rbp_core::TRAINING_LOG_INTERVAL {
            self.flush();
            let secs = prior.0.elapsed().as_secs().max(1) as f64;
            let curr = self.infos();
            let rate = (curr - prior.1) as f64 / secs;
            *prior = (Instant::now(), curr);
            Some(format!(
                "{:<20}{:<20}{:<20}{:<20}",
                format!("epoch {}", self.epoch()),
                format!("nodes {}", self.nodes()),
                format!("infos {}", curr),
                format!("I/sec {:.1}", rate),
            ))
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
