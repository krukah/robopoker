//! One Elkan iteration's worth of work — the unit yielded by `Kmeans`.

use crate::Drift;
use std::time::Duration;

/// Bundles everything a consumer needs to make decisions per step: the drift (for convergence
/// checks), the wall-clock duration of `step_elkan`, the iteration index, a `last` flag, a
/// `frozen` flag set when the configured drift threshold triggers early termination, and the
/// per-cluster point counts at this step.
pub struct Step<const K: usize> {
    /// 0-based iteration index.
    index: usize,
    /// Centroid movement produced by this step.
    drift: Drift<K>,
    /// Wall-clock time spent inside `step_elkan` (and bounds update).
    elapsed: Duration,
    /// True if `with_bound(t)` was set and this step is the t-th yield,
    /// or the freeze threshold collapsed the bound to here.
    last: bool,
    /// Set when drift fell below the freeze threshold, so the next `next()`
    /// returns `None`. Read it to fire termination side effects.
    frozen: bool,
    /// Per-cluster point counts at this step. Computed by walking
    /// the iterator's owned bounds — no extra EMD work.
    sizes: [u64; K],
    /// Fraction in `[0.0, 1.0]` of points that changed cluster since the last
    /// iteration. Goes to zero at convergence; the complement of `drift`.
    reassignment: f64,
}

impl<const K: usize> Step<K> {
    pub fn new(
        index: usize,
        drift: Drift<K>,
        elapsed: Duration,
        last: bool,
        frozen: bool,
        sizes: [u64; K],
        reassignment: f64,
    ) -> Self {
        Self {
            index,
            drift,
            elapsed,
            last,
            frozen,
            sizes,
            reassignment,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn drift(&self) -> &Drift<K> {
        &self.drift
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn last(&self) -> bool {
        self.last
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }

    pub fn sizes(&self) -> &[u64; K] {
        &self.sizes
    }

    pub fn reassignment(&self) -> f64 {
        self.reassignment
    }
}
