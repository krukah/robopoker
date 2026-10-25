//! Iterator that drives Elkan k-means iterations as a stream of `Step<K>`.
//!
//! Each yielded step carries drift, elapsed time, and freeze detection, so the
//! consumer tracks none of it itself.

use crate::Drift;
use crate::Elkan;
use crate::Histogram;
use crate::Layer;
use crate::Prune;
use crate::Step;
use elkan::Prior;
use pokerkit::Energy;
use std::time::Instant;

/// Streams Elkan iterations as `Step<K>` values. Created by `Layer::kmeans()`. Owns the bounds
/// buffer for its lifetime — takes `layer.bounds` at construction and gives it back on drop, so
/// the layer is consistent whether the loop completes, breaks, or panics.
pub struct Kmeans<'street, const K: usize, const N: usize, B> {
    layer: &'street mut Layer<K, N, B>,
    bounds: Box<[B]>,
    /// Snapshot of previous-iter cluster assignments — see `Prior`.
    prior: Prior<N>,
    index: usize,
    total: Option<usize>,
    threshold: Option<Energy>,
}

impl<'street, const K: usize, const N: usize, B> Kmeans<'street, K, N, B>
where
    B: Prune<K>,
{
    pub(crate) fn new(layer: &'street mut Layer<K, N, B>) -> Self {
        let bounds = std::mem::take(layer.boundings_mut());
        let prior = Prior::from_bounds((&*bounds).try_into().expect("N"));
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
        *self.total_mut() = Some(total);
        self
    }

    /// Stop iterating when the largest centroid movement falls below
    /// `threshold`. The frozen step is yielded once with `frozen =
    /// true`; subsequent `next()` returns `None`. Set to `0.0` (or
    /// omit) to disable early termination.
    pub fn with_threshold(mut self, threshold: Energy) -> Self {
        *self.threshold_mut() = (threshold > 0.0).then_some(threshold);
        self
    }

    /// Assigns every point to its nearest final centroid. The last step moved the centroids
    /// after assigning, so its assignments are one update stale until this runs.
    pub fn settle(&mut self) {
        let (layer, bounds, _) = self.parts();
        layer.assign(bounds);
    }

    /// The layer, the bounds it lends out while this iterator runs, and the prior snapshot,
    /// borrowed disjointly.
    fn parts(&mut self) -> (&Layer<K, N, B>, &mut [B; N], &mut Prior<N>) {
        let Self {
            layer, bounds, prior, ..
        } = self;
        (layer, (&mut **bounds).try_into().expect("N"), prior)
    }

    fn step(&mut self) -> ([Histogram; K], Drift<K>) {
        let (layer, bounds, _) = self.parts();
        layer.step_elkan(bounds)
    }

    fn tally(&mut self) -> ([u64; K], f64) {
        let (_, bounds, prior) = self.parts();
        prior.tally(bounds)
    }
}

impl<const K: usize, const N: usize, B> Kmeans<'_, K, N, B> {
    fn layer_mut(&mut self) -> &mut Layer<K, N, B> {
        self.layer
    }

    fn bounds_mut(&mut self) -> &mut Box<[B]> {
        &mut self.bounds
    }

    fn index(&self) -> usize {
        self.index
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }

    fn total(&self) -> Option<usize> {
        self.total
    }

    fn total_mut(&mut self) -> &mut Option<usize> {
        &mut self.total
    }

    fn threshold(&self) -> Option<Energy> {
        self.threshold
    }

    fn threshold_mut(&mut self) -> &mut Option<Energy> {
        &mut self.threshold
    }
}

impl<const K: usize, const N: usize, B> Drop for Kmeans<'_, K, N, B> {
    fn drop(&mut self) {
        *self.layer_mut().boundings_mut() = std::mem::take(self.bounds_mut());
    }
}

impl<const K: usize, const N: usize, B> Iterator for Kmeans<'_, K, N, B>
where
    B: Prune<K>,
{
    type Item = Step<K>;

    fn next(&mut self) -> Option<Step<K>> {
        if matches!(self.total(), Some(t) if self.index() >= t) {
            return None;
        }
        let t = Instant::now();
        let (centroids, drift) = self.step();
        **self.layer_mut().centroids_mut() = centroids;
        let elapsed = t.elapsed();
        let index = self.index();
        let frozen = matches!(self.threshold(), Some(thr) if drift.frozen(thr));
        // Freeze collapses the bound to the current step, so `last` reports no further
        // iterations and the next `next()` returns None via the `index >= total` guard.
        if frozen {
            *self.total_mut() = Some(index + 1);
        }
        let last = matches!(self.total(), Some(t) if index + 1 >= t);
        let (sizes, reassignment) = self.tally();
        *self.index_mut() += 1;
        Some(Step::new(index, drift, elapsed, last, frozen, sizes, reassignment))
    }
}
