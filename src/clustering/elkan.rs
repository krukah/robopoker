use super::*;
use crate::Energy;
use rayon::prelude::*;

/// Trait for Elkan's K-Means algorithm. This exploits the triangle inequality to accelerate
/// computation of a naive O(n * k * t) algorithm that recalculates centroid-to-point distances
/// on every iteration. The key insight of Elkan (2003) is to use the triangle inequality to
/// avoid recalculating distances between points and centroids that are already known to be
/// far apart.
///
/// We also provide a naive implementation for benchmarking purposes. However, our sampling of
/// arbitrary Histograms is biased toward the naive implementation: Elkan's optimization assumes that
/// the algorithm runtime is dominated by the EMD distance calculations. This is true for the abstract
/// EMD distance calculations between distributions over Abstraactions, but not necessarily true for the
/// EMD distance calculations between distributions over River Equity. Still, with large enough N, the
/// memory overhead of Elkan's algorithm is worth the shorter runtime.
///
/// See benchmarks.rs, sinkhorn.rs for more information.
pub trait Elkan<const K: usize, const N: usize>: Sync {
    type P: Absorb + Copy + Sync + Send;

    fn points(&self) -> &[Self::P; N];
    fn kmeans(&self) -> &[Self::P; K];
    fn bounds(&self) -> &[Bounds<K>; N];

    fn init_kmeans(&self) -> [Self::P; K];
    fn init_bounds(&self) -> Box<[Bounds<K>; N]> {
        (0..N)
            .into_par_iter()
            .map(|i| self.neighbor(i))
            .map(Bounds::from)
            .collect::<Vec<_>>()
            .try_into()
            .expect("bounds.len() == N")
    }

    fn distance(&self, h1: &Self::P, h2: &Self::P) -> Energy;

    fn point(&self, i: usize) -> &Self::P {
        &self.points()[i]
    }
    fn kmean(&self, j: usize) -> &Self::P {
        &self.kmeans()[j]
    }
    fn bound(&self, i: usize) -> &Bounds<K> {
        &self.bounds()[i]
    }

    fn t(&self) -> usize {
        1024
    }

    /// Compute the nearest neighbor in O(k) * MetricCost
    fn neighbor(&self, i: usize) -> (usize, f32) {
        let ref x = self.point(i);
        self.kmeans()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, self.distance(c, x)))
            .inspect(|(_, d)| assert!(d.is_finite()))
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
            .unwrap()
    }

    /// Compute d(c, c') for all centers c and c'
    fn pairwises(&self) -> [[f32; K]; K] {
        use rayon::prelude::*;
        (0..K)
            .into_par_iter()
            .map(|i| std::array::from_fn(|j| self.pairwise(i, j)))
            .collect::<Vec<_>>()
            .try_into()
            .expect("K rows")
    }

    fn pairwise(&self, i: usize, j: usize) -> f32 {
        if i == j {
            0.0
        } else {
            self.distance(self.kmean(i), self.kmean(j))
        }
    }

    /// Compute s(c) = (1/2) min_{c'!=c} d(c, c')
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

    fn drift(&self, news: &[Self::P]) -> [f32; K] {
        std::array::from_fn(|i| self.distance(&news[i], self.kmean(i)))
    }

    /// Refresh stale upper bound before checking triangle inequality
    fn refresh(&self, b: &mut Bounds<K>, x: &Self::P) {
        if b.stale() {
            b.refresh(self.distance(x, self.kmean(b.j())));
        }
    }
    /// Update bound for a single point-centroid pair
    /// Check triangle inequality and maybe reassign
    fn rebound(&self, b: &mut Bounds<K>, j: usize, metric: &[[f32; K]; K], x: &Self::P) {
        if b.has_shifted(metric, j) {
            b.witness(self.distance(x, self.kmean(j)), j);
        }
    }

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

    /// Compute new centroids from assigned points.
    /// Mutates bounds in place to avoid cloning.
    /// Step 1: Update bounds and reassign points (using OLD centroids from self.kmeans())
    /// Step 2: Compute NEW centroids based on UPDATED assignments
    /// Step 3: Compute drift between old and new centroids
    /// Step 4: Shift bounds for next iteration
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

    /// without optimization
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
