use super::*;
use crate::Energy;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

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
pub trait Elkan: Sync {
    type P: Absorb + Clone + Sync;

    fn distance(&self, h1: &Self::P, h2: &Self::P) -> Energy;

    fn dataset(&self) -> &Vec<Self::P>;
    fn kmeans(&self) -> &Vec<Self::P>;
    fn bounds(&self) -> &Vec<Bound>;

    fn init_kmeans(&self) -> Vec<Self::P>;
    fn init_bounds(&self) -> Vec<Bound> {
        let n = self.n();
        let k = self.k();
        (0..n)
            .into_iter()
            .map(|i| self.neighbor(i))
            .map(|(j, dist)| Bound::new(j, k, dist))
            .collect::<Vec<_>>()
    }

    fn t(&self) -> usize {
        1024
    }
    fn k(&self) -> usize {
        self.kmeans().len()
    }
    fn n(&self) -> usize {
        self.dataset().len()
    }

    fn point(&self, i: usize) -> &Self::P {
        self.dataset().get(i).expect("n points")
    }
    fn kmean(&self, j: usize) -> &Self::P {
        self.kmeans().get(j).expect("k means")
    }
    fn bound(&self, i: usize) -> &Bound {
        self.bounds().get(i).expect("n bounds")
    }

    /// Compute the nearest neighbor in O(k) * MetricCost
    fn neighbor(&self, i: usize) -> (usize, f32) {
        // @parallelizable
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
    fn pairwise(&self) -> Vec<Vec<f32>> {
        // @parallelizable
        self.kmeans()
            .iter()
            .flat_map(|c1| self.kmeans().iter().map(|c2| self.distance(c1, c2)))
            .collect::<Vec<_>>()
            .chunks(self.k())
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>()
    }

    /// Compute s(c) = (1/2) min_{c'!=c} d(c, c')
    fn midpoints(&self) -> Vec<f32> {
        // @parallelizable
        self.pairwise()
            .iter()
            .enumerate()
            .map(|(i, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, &d)| d)
                    .reduce(f32::min)
                    .map(|d| d * 0.5)
                    .unwrap()
            })
            .collect::<Vec<_>>()
    }

    /// Identify points where u(x) <= s(c(x))
    fn excluded(&self) -> HashSet<usize> {
        // @parallelizable
        let ref midpoints = self.midpoints();
        self.bounds()
            .par_iter()
            .enumerate()
            .filter(|(_, b)| b.u() <= midpoints[b.j()])
            .map(|(x, _)| x)
            .collect::<HashSet<_>>()
    }

    /// Identify points where u(x) > s(c(x)) requiring bound updates
    fn included(&self) -> HashMap<usize, (&Self::P, Bound)> {
        // @parallelizable
        let ref excluded = self.excluded();
        (0..self.n())
            .filter(|i| !excluded.contains(i))
            .map(|i| (i, (self.point(i), self.bound(i))))
            .map(|(i, (p, b))| (i, (p, b.clone())))
            .collect::<HashMap<_, _>>()
    }

    /// Step 3: Update bounds for each point/center pair using triangle inequality
    fn triangle(&self) -> HashMap<usize, (&Self::P, Bound)> {
        // @parallelizable
        let ref pairwise = self.pairwise();
        let mut included = self.included();
        (0..self.k()).for_each(|j| {
            included
                .par_iter_mut()
                .for_each(|(_, (x, b))| self.rebound(b, j, pairwise, x));
        });
        included
    }

    fn drift(&self, news: &[Self::P]) -> Vec<Energy> {
        // @parallelizable
        self.kmeans()
            .iter()
            .zip(news.iter())
            .map(|(old, new)| self.distance(new, old))
            .collect::<Vec<_>>()
    }

    /// Update bound for a single point-centroid pair
    fn rebound(&self, b: &mut Bound, j: usize, metric: &[Vec<f32>], x: &Self::P) {
        // Refresh stale upper bound if needed
        // Check triangle inequality and maybe reassign
        if b.stale() {
            b.refresh(self.distance(x, self.kmean(b.j())));
        }
        if b.moved(metric, j) {
            b.witness(self.distance(x, self.kmean(j)), j);
        }
    }

    /// Merge updated bounds back with original
    fn next_elkan_bounds(&self) -> Vec<Bound> {
        // @parallelizable
        let n = self.n();
        let ref tri = self.triangle();
        (0..n)
            .into_par_iter()
            .map(|i| tri.get(&i).map(|(_, b)| b).unwrap_or_else(|| self.bound(i)))
            .cloned()
            .collect::<Vec<_>>()
    }

    fn next_elkan_kmeans(&self, bounds: &[Bound]) -> Vec<Self::P> {
        // @parallelizable
        let k = self.k();
        (0..k)
            .map(|j| {
                bounds
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.j == j)
                    .map(|(i, _)| self.point(i))
                    .fold(Self::P::default(), Self::P::absorb)
            })
            .collect::<Vec<_>>()
    }

    /// Compute new centroids from assigned points
    fn next_eklan(&self) -> (Vec<Self::P>, Vec<Bound>) {
        // @parallelizable
        // Step 1: Update bounds and reassign points (using OLD centroids from self.kmeans())
        // Step 2: Compute NEW centroids based on UPDATED assignments
        // Step 3: Compute drift between old and new centroids
        // Step 4: Shift bounds for next iteration
        let bounds = self.next_elkan_bounds();
        let kmeans = self.next_elkan_kmeans(&bounds);
        let ref drift = self.drift(&kmeans);
        let bounds = bounds
            .into_par_iter()
            .map(|b| b.shift(drift))
            .collect::<Vec<_>>();
        (kmeans, bounds)
    }

    /// without optimization
    fn next_naive(&self) -> (Vec<Self::P>, Vec<Bound>) {
        let n = self.n();
        let k = self.k();
        let mut kmeans = (0..k).map(|_| Self::P::default()).collect::<Vec<_>>();
        let mut bounds = (0..n).map(|j| Bound::new(j, k, 0.0)).collect::<Vec<_>>();
        for (i, (j, distance)) in (0..n)
            .into_par_iter()
            .map(|i| (i, self.neighbor(i)))
            .collect::<Vec<_>>()
            .into_iter()
        {
            bounds.get_mut(i).expect("n bounds").assign(j, distance);
            kmeans.get_mut(j).expect("k bounds").engulf(self.point(i));
        }
        (kmeans, bounds)
    }

    fn rms(&self) -> Energy {
        // @parallelizable
        (self
            .bounds()
            .par_iter()
            .enumerate()
            .map(|(i, b)| self.distance(self.point(i), self.kmean(b.j())))
            .map(|d| d * d)
            .sum::<Energy>()
            / self.n() as Energy)
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::Elkan;
    use super::TurnLayer;

    #[test]
    #[ignore]
    /// This test is kinda fuzzy. There's no guarantee that we achieve strict monotonicity.
    /// Techincally, we don't even guarantee that the RMS will not increase. This is because
    /// there is some probability that a cluster will end up empty, after which we will replace it
    /// with a random point (see Self::heal()), after which this new random point may introduce a larger error.
    /// I don't know, haven't done any paper pencil proofing. But this fuzziness is fine for me. Really the important
    /// thing is that the algorithm is equivalent to the naive KMeans algorithm.
    fn elkan_rms_decreases() {
        let mut km = TurnLayer::new();
        let mut rms = vec![km.rms()];
        for _ in 0..km.t() {
            km.step_elkan();
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
        let mut km = TurnLayer::new();
        for _ in 0..km.t() {
            km.step_elkan();
        }
        let r1 = km.rms();
        km.step_elkan();
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
        let km = TurnLayer::new();
        let mut elkan = km.clone();
        let mut naive = km.clone();
        for _ in 0..km.t() {
            elkan.step_elkan();
            naive.step_naive();
            assert_eq!(elkan.rms(), naive.rms());
            assert_eq!(elkan.kmeans(), naive.kmeans());
        }
    }
}
