//! Snapshot of per-point cluster assignments — used by `Kmeans` to
//! compute reassignment rate per iteration without re-walking bounds
//! twice.
//!
//! Stored as `u8` since cluster IDs are bounded by
//! `KMEANS_MAX_CLUSTER_COUNT` =
//! 255. Const-assertions on the per-street K constants enforce this
//! at compile time, so the `j as u8` cast in `tally` cannot truncate.

use crate::Bounds;

/// Per-point assignment snapshot, indexed by point index `i`. Holds
/// cluster IDs from the previous iteration so `tally` can detect
/// which points moved.
pub struct Prior<const N: usize>(Box<[u8; N]>);

impl<const N: usize> Prior<N> {
    /// Seed from the post-init bounds — first iter's reassignment
    /// rate is then "what changed during the first step", not
    /// "everything moved from cluster 0".
    pub fn from_bounds<const K: usize>(bounds: &[Bounds<K>; N]) -> Self {
        let inner: Box<[u8; N]> = bounds
            .iter()
            .map(|b| b.j() as u8)
            .collect::<Vec<_>>()
            .try_into()
            .expect("N");
        Self(inner)
    }

    /// Single O(N) pass over `bounds`: tallies cluster sizes, counts
    /// points whose assignment changed since the last call, updates
    /// the snapshot inline. Returns `(sizes, reassignment_rate)`
    /// where `reassignment_rate` is in `[0.0, 1.0]`.
    pub fn tally<const K: usize>(&mut self, bounds: &[Bounds<K>; N]) -> ([u64; K], f64) {
        let mut sizes = [0u64; K];
        let mut moved = 0u64;
        bounds.iter().enumerate().for_each(|(i, b)| {
            let j = b.j();
            sizes[j] += 1;
            if (j as u8) != self.0[i] {
                moved += 1;
                self.0[i] = j as u8;
            }
        });
        (sizes, moved as f64 / N as f64)
    }
}
