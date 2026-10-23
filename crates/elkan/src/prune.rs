use crate::Drift;

/// Per-point bookkeeping that lets k-means skip distance calls without changing which centroid
/// a point lands on. [`Bounds`](crate::Bounds) keeps a lower bound per centroid (Elkan, K floats
/// per point); [`Hamerly`](crate::Hamerly) keeps one (a constant 16 bytes per point) at the cost
/// of more distance calls.
pub trait Prune<const K: usize>: Clone + Default + std::fmt::Debug + Send + Sync + From<(usize, f32)> {
    /// Currently assigned centroid index.
    fn j(&self) -> usize;
    /// Reassigns to the nearest centroid, calling `distance(j)` = d(x, c_j) only where the
    /// bounds cannot rule centroid `j` out.
    fn settle<F>(&mut self, pairwise: &[[f32; K]; K], midpoints: &[f32; K], distance: F)
    where
        F: Fn(usize) -> f32;
    /// Loosens the bounds by how far each centroid moved.
    fn update(&mut self, drift: &Drift<K>);
}
