//! First-class wrapper around per-iteration centroid movement.
//!
//! `Drift<K>` is what `Elkan::step_elkan` produces alongside the new centroids. Each entry is
//! the distance from `old_kmeans[j]` to `new_kmeans[j]` — Elkan's bound update needs these
//! distances anyway, so reading them is free.

use crate::Energy;

/// Per-cluster centroid movement after one Elkan iteration.
#[derive(Clone, Debug)]
pub struct Drift<const K: usize>([Energy; K]);

impl<const K: usize> Drift<K> {
    pub fn from_array(drifts: [Energy; K]) -> Self {
        Self(drifts)
    }

    /// Largest single-cluster centroid movement. Canonical convergence signal.
    pub fn max(&self) -> Energy {
        self.0.iter().copied().fold(0.0, Energy::max)
    }

    /// Largest movement among every centroid but `j`.
    pub fn max_except(&self, j: usize) -> Energy {
        self.0
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != j)
            .map(|(_, d)| *d)
            .fold(0.0, Energy::max)
    }

    /// Smallest single-cluster centroid movement.
    pub fn min(&self) -> Energy {
        self.0.iter().copied().fold(Energy::INFINITY, Energy::min)
    }

    /// Average centroid movement across all K clusters.
    pub fn mean(&self) -> Energy {
        self.0.iter().sum::<Energy>() / K as Energy
    }

    /// True when the largest centroid movement is below the freeze threshold. Used as an early-
    /// termination signal: if no centroid has moved more than `threshold` this iteration, the
    /// algorithm is at (or near) a fixed point and further iteration is wasted work.
    pub fn frozen(&self, threshold: Energy) -> bool {
        self.max() < threshold
    }

    /// Direct array view — the form `Bounds::update` consumes.
    pub fn as_array(&self) -> &[Energy; K] {
        &self.0
    }
}

impl<const K: usize> IntoIterator for Drift<K> {
    type Item = Energy;
    type IntoIter = std::array::IntoIter<Energy, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'drift, const K: usize> IntoIterator for &'drift Drift<K> {
    type Item = &'drift Energy;
    type IntoIter = std::slice::Iter<'drift, Energy>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
