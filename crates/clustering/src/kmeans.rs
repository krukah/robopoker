//! Iterator that drives Elkan k-means iterations as a stream of `Step<K>`.
//!
//! `Layer::kmeans()` returns a `Kmeans` — built fluently with optional
//! `with_bound` / `with_threshold`. Each step yielded carries the
//! drift, elapsed time, and freeze detection so the consumer doesn't
//! have to track any of that itself.

use crate::Bounds;
use crate::Elkan;
use crate::Layer;
use crate::Step;
use crate::prior::Prior;
use rbp_core::Energy;
use std::time::Instant;

/// Streams Elkan iterations as `Step<K>` values. Created by
/// `Layer::kmeans()`. Owns the bounds buffer for its lifetime — swaps
/// `layer.bounds` out at construction and restores via pointer swap on
/// drop, so the layer is consistent whether the loop completes,
/// breaks, or panics.
///
/// Built fluently:
/// ```text
/// let ref mut iter = layer.kmeans()
///     .with_bound(layer.t())          // bound the stream
///     .with_threshold(1e-5);          // early-terminate at convergence
/// for step in iter { … }
/// ```
pub struct Kmeans<'street, const K: usize, const N: usize> {
    layer: &'street mut Layer<K, N>,
    bounds: Box<[Bounds<K>; N]>,
    /// Snapshot of previous-iter cluster assignments — see `Prior`.
    prior: Prior<N>,
    index: usize,
    total: Option<usize>,
    threshold: Option<Energy>,
}

impl<'street, const K: usize, const N: usize> Kmeans<'street, K, N> {
    pub(crate) fn new(layer: &'street mut Layer<K, N>) -> Self {
        let placeholder: Box<[Bounds<K>; N]> = vec![Bounds::default(); N].try_into().expect("N");
        let bounds = std::mem::replace(layer.boundings_mut(), placeholder);
        let prior = Prior::from_bounds(&bounds);
        Self {
            layer,
            bounds,
            prior,
            index: 0,
            total: None,
            threshold: None,
        }
    }

    /// Cap the iterator at `total` steps. Without this the iterator
    /// runs forever; with it, `Step::last` is set on the t-th yield.
    pub fn with_bound(mut self, total: usize) -> Self {
        self.total = Some(total);
        self
    }

    /// Stop iterating when the largest centroid movement falls below
    /// `threshold`. The frozen step is yielded once with `frozen =
    /// true`; subsequent `next()` returns `None`. Set to `0.0` (or
    /// omit) to disable early termination.
    pub fn with_threshold(mut self, threshold: Energy) -> Self {
        self.threshold = (threshold > 0.0).then_some(threshold);
        self
    }
}

impl<const K: usize, const N: usize> Drop for Kmeans<'_, K, N> {
    fn drop(&mut self) {
        // Pointer swap — the iterator's worked-on box ends up back in
        // layer.bounds; the layer's placeholder ends up in self.bounds
        // and is freed when Kmeans drops. No allocation.
        std::mem::swap(&mut self.bounds, self.layer.boundings_mut());
    }
}

impl<const K: usize, const N: usize> Iterator for Kmeans<'_, K, N> {
    type Item = Step<K>;
    fn next(&mut self) -> Option<Step<K>> {
        if matches!(self.total, Some(t) if self.index >= t) {
            return None;
        }
        let t = Instant::now();
        let (centroids, drift) = (*self.layer).step_elkan(&mut self.bounds);
        **self.layer.centroids_mut() = centroids;
        let elapsed = t.elapsed();
        let index = self.index;
        let frozen = matches!(self.threshold, Some(thr) if drift.frozen(thr));
        // Freeze collapses the bound to the current step — `last` then
        // reflects that no more iterations follow, and the next `next()`
        // returns None via the `index >= total` guard above.
        if frozen {
            self.total = Some(index + 1);
        }
        let last = matches!(self.total, Some(t) if index + 1 >= t);
        let (sizes, reassignment) = self.prior.tally(&self.bounds);
        self.index += 1;
        Some(Step {
            index,
            drift,
            elapsed,
            last,
            frozen,
            sizes,
            reassignment,
        })
    }
}
