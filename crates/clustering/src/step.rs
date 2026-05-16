//! One Elkan iteration's worth of work — the unit yielded by `Kmeans`.

use crate::Drift;
use std::time::Duration;

/// Bundles everything a consumer needs to make decisions per step:
/// the drift (for convergence checks), the wall-clock duration of
/// `step_elkan`, the iteration index, a `last` flag, a `frozen` flag
/// set when the configured drift threshold triggers early termination,
/// and the per-cluster point counts at this step.
///
/// `sizes` is computed every step (~O(N) integer increments, free
/// relative to step_elkan's EMD work) so the consumer can plot the
/// trajectory of the size distribution as clustering converges.
pub struct Step<const K: usize> {
    /// 0-based iteration index.
    pub index: usize,
    /// Centroid movement produced by this step.
    pub drift: Drift<K>,
    /// Wall-clock time spent inside `step_elkan` (and bounds update).
    pub elapsed: Duration,
    /// True if `with_bound(t)` was set and this step is the t-th yield,
    /// or the freeze threshold collapsed the bound to here.
    pub last: bool,
    /// True when this step's drift fell below the configured freeze
    /// threshold — the iterator will return `None` from the next
    /// `next()` call. The consumer can read `frozen` to fire
    /// termination side effects.
    pub frozen: bool,
    /// Per-cluster point counts at this step. Computed by walking
    /// the iterator's owned bounds — no extra EMD work.
    pub sizes: [u64; K],
    /// Fraction in `[0.0, 1.0]` of points whose assigned cluster
    /// changed between the previous iteration and this one. Goes to
    /// zero at convergence; a complement to `drift` (centroids stop
    /// moving).
    pub reassignment: f64,
}
