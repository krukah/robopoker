use super::*;
use rayon::prelude::*;
use rbp_core::Energy;

/// Triangle-inequality accelerated k-means clustering.
///
/// Implements Elkan (2003) to reduce the O(N × K × T) naive algorithm's
/// distance computations. By maintaining upper/lower bounds on point-centroid
/// distances, we can skip most EMD calculations while guaranteeing identical
/// results to naive k-means.
///
/// # Complexity
///
/// - Naive: O(N × K × T × D) where D is EMD cost
/// - Elkan: O(N × K × T × D / prune_factor) with ~10-100x speedup typical
///
/// # Implementation
///
/// - `step_elkan()` — Single iteration with bound maintenance
/// - `step_naive()` — Reference implementation for verification
/// - `init_kmeans()` — K-means++ initialization for better convergence
///
/// # Type Parameters
///
/// - `K` — Number of clusters (compile-time constant)
/// - `N` — Number of data points (compile-time constant)
pub trait Elkan<const K: usize, const N: usize>: Sync {
    /// Point type that can be absorbed into centroids.
    type P: Absorb + Copy + Sync + Send;
    /// Returns the data points to cluster.
    fn points(&self) -> &[Self::P; N];
    /// Returns current centroid positions.
    fn kmeans(&self) -> &[Self::P; K];
    /// Returns per-point distance bounds.
    fn bounds(&self) -> &[Bounds<K>; N];
    /// Initializes centroids (typically k-means++).
    fn init_kmeans(&self) -> [Self::P; K];
    /// Initializes bounds by computing all point-centroid distances.
    fn init_bounds(&self) -> Box<[Bounds<K>; N]> {
        (0..N)
            .into_par_iter()
            .map(|i| self.neighbor(i))
            .map(Bounds::from)
            .collect::<Vec<_>>()
            .try_into()
            .expect("bounds.len() == N")
    }

    /// Computes distance between two points (e.g., EMD).
    fn distance(&self, h1: &Self::P, h2: &Self::P) -> Energy;
    /// Gets point by index.
    fn point(&self, i: usize) -> &Self::P {
        &self.points()[i]
    }
    /// Gets centroid by index.
    fn kmean(&self, j: usize) -> &Self::P {
        &self.kmeans()[j]
    }
    /// Gets bounds for point by index.
    fn bound(&self, i: usize) -> &Bounds<K> {
        &self.bounds()[i]
    }
    /// Number of iterations to run.
    fn t(&self) -> usize {
        1024
    }
    /// Finds nearest centroid for a point (O(K) distance calls).
    fn neighbor(&self, i: usize) -> (usize, f32) {
        let ref x = self.point(i);
        self.kmeans()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, self.distance(c, x)))
            .inspect(|(_, d)| debug_assert!(d.is_finite()))
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
            .unwrap()
    }

    /// Computes pairwise distances between all centroids.
    fn pairwises(&self) -> [[f32; K]; K] {
        use rayon::prelude::*;
        (0..K)
            .into_par_iter()
            .map(|i| std::array::from_fn(|j| self.pairwise(i, j)))
            .collect::<Vec<_>>()
            .try_into()
            .expect("K rows")
    }

    /// Computes distance between two centroids.
    fn pairwise(&self, i: usize, j: usize) -> f32 {
        if i == j {
            0.0
        } else {
            self.distance(self.kmean(i), self.kmean(j))
        }
    }

    /// Computes s(c) = (1/2) min_{c'≠c} d(c, c') for each centroid.
    fn midpoints(&self, pairwise: &[[f32; K]; K]) -> [f32; K] {
        let mut result = [f32::MAX; K];
        pairwise.iter().enumerate().for_each(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .for_each(|(_, &d)| result[i] = result[i].min(d * 0.5))
        });
        result
    }

    /// Computes how far each centroid moved this iteration.
    fn drift(&self, news: &[Self::P]) -> [f32; K] {
        std::array::from_fn(|i| self.distance(&news[i], self.kmean(i)))
    }

    /// Refreshes stale upper bound before triangle inequality check.
    fn refresh(&self, b: &mut Bounds<K>, x: &Self::P) {
        if b.stale() {
            b.refresh(self.distance(x, self.kmean(b.j())));
        }
    }
    /// Updates bound for point-centroid pair, possibly reassigning.
    fn rebound(&self, b: &mut Bounds<K>, j: usize, metric: &[[f32; K]; K], x: &Self::P) {
        if b.has_shifted(metric, j) {
            b.witness(self.distance(x, self.kmean(j)), j);
        }
    }

    /// Computes new centroids from current assignments.
    fn centroids(&self, bounds: &[Bounds<K>]) -> [Self::P; K] {
        std::array::from_fn(|j| {
            bounds
                .iter()
                .enumerate()
                .filter(|(_, b)| b.j() == j)
                .map(|(i, _)| self.point(i))
                .fold(self.kmean(j).identity(), Self::P::absorb)
        })
    }

    /// Executes one Elkan iteration with bound maintenance.
    ///
    /// 1. Update bounds and reassign points using current centroids
    /// 2. Compute new centroids from updated assignments
    /// 3. Compute drift (how far each centroid moved)
    /// 4. Shift bounds to account for centroid movement
    fn step_elkan(&self, bounds: &mut [Bounds<K>; N]) -> [Self::P; K] {
        let pairwise = self.pairwises();
        let midpoints = self.midpoints(&pairwise);
        bounds
            .par_iter_mut()
            .enumerate()
            .filter(|(_, b)| b.u() > midpoints[b.j()])
            .for_each(|(i, b)| {
                self.refresh(b, self.point(i));
                (0..K).for_each(|j| self.rebound(b, j, &pairwise, self.point(i)))
            });
        let kmeans = self.centroids(bounds);
        let drifts = self.drift(&kmeans);
        bounds.par_iter_mut().for_each(|b| b.update(&drifts));
        kmeans
    }

    /// Executes one naive iteration (for verification/benchmarking).
    fn step_naive(&self) -> [Self::P; K] {
        let identity = self.kmean(0).identity();
        let assignments = (0..N)
            .into_par_iter()
            .map(|i| self.neighbor(i).0)
            .collect::<Vec<usize>>();
        std::array::from_fn(|j| {
            assignments
                .iter()
                .enumerate()
                .filter(|(_, k)| k == &&j)
                .map(|(i, _)| self.point(i))
                .fold(identity, Self::P::absorb)
        })
    }

    /// Computes root-mean-square error (convergence metric).
    fn rms(&self) -> Energy {
        (self
            .bounds()
            .par_iter()
            .enumerate()
            .map(|(i, b)| self.distance(self.point(i), self.kmean(b.j())))
            .map(|d| d * d)
            .sum::<Energy>()
            / N as Energy)
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::Elkan;
    use super::TestLayer;

    #[test]
    #[ignore]
    /// This test is kinda fuzzy. There's no guarantee that we achieve strict monotonicity.
    /// Techincally, we don't even guarantee that the RMS will not increase. This is because
    /// there is some probability that a cluster will end up empty, after which we will replace it
    /// with a random point (see Self::heal()), after which this new random point may introduce a larger error.
    /// I don't know, haven't done any paper pencil proofing. But this fuzziness is fine for me. Really the important
    /// thing is that the algorithm is equivalent to the naive KMeans algorithm.
    fn elkan_rms_decreases() {
        let mut km = TestLayer::new();
        let mut rms = vec![km.rms()];
        for _ in 0..km.t() {
            km.step();
            rms.push(km.rms());
        }
        for window in rms.windows(2) {
            assert!(
                window[0] >= window[1],
                "RMS increasing: {} -> {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    #[ignore]
    /// This test is kinda fuzzy. There's no guarantee that this must converge within
    /// Self::t() iterations. And the scale of this distance/energy values are unitless,
    /// so the arbitrary 1/100 threshold is meaningless. But it seems to pass most times so whatever.
    fn elkan_rms_converges() {
        let mut km = TestLayer::new();
        for _ in 0..km.t() {
            km.step();
        }
        let r1 = km.rms();
        km.step();
        let r2 = km.rms();
        assert!(
            (r1 - r2).abs() <= 0.01,
            "RMS is unlikely large: {} -> {}",
            r1,
            r2
        );
    }

    #[test]
    #[ignore]
    /// Test that Elkan's algorithm is equivalent to the naive implementation.
    /// The optimization is not an approximation, so we can assert that the
    /// state machine of the algorithm is identical to the naive implementation at every iteration.
    fn elkan_naive_equivalence() {
        let km = TestLayer::new();
        let mut elkan = km.clone();
        let mut naive = km.clone();
        for _ in 0..km.t() {
            elkan.step();
            naive.naive();
            assert_eq!(elkan.rms(), naive.rms());
            assert_eq!(elkan.kmeans(), naive.kmeans());
        }
    }
}
