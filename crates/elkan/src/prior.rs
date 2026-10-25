//! Each point's cluster as of the last iteration. `u8` holds it because cluster ids stay below
//! `KMEANS_MAX_CLUSTER_COUNT` = 256, so the casts in `tally` cannot truncate.

use crate::Prune;

/// Per-point assignment snapshot, indexed by point.
pub struct Prior<const N: usize>(Box<[u8; N]>);

impl<const N: usize> Prior<N> {
    /// Seeds from the post-init bounds, so the first reassignment rate counts what the first step
    /// changed rather than every point leaving cluster 0.
    pub fn from_bounds<const K: usize, B>(bounds: &[B; N]) -> Self
    where
        B: Prune<K>,
    {
        let inner: Box<[u8; N]> = bounds
            .iter()
            .map(|b| b.j() as u8)
            .collect::<Vec<_>>()
            .try_into()
            .expect("N");
        Self(inner)
    }

    /// One pass: cluster sizes and the share of points whose cluster changed since the last call,
    /// in `[0, 1]`. Updates the snapshot.
    pub fn tally<const K: usize, B>(&mut self, bounds: &[B; N]) -> ([u64; K], f64)
    where
        B: Prune<K>,
    {
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
