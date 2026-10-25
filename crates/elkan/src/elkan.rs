use super::*;
use crate::Energy;
use rayon::prelude::*;

/// Triangle-inequality accelerated k-means (Elkan 2003), over `K` clusters and
/// `N` points. Upper/lower bounds on point-centroid distances let it skip most
/// of the naive O(N × K × T × D) distance calls — typically 10-100× fewer —
/// while producing results identical to naive k-means.
pub trait Elkan<const K: usize, const N: usize>: Sync {
    /// Point type that can be absorbed into centroids.
    type P: Absorb + Copy + Sync + Send;
    /// Per-point bounds that prune distance calls.
    type B: Prune<K>;
    /// Returns the data points to cluster.
    fn points(&self) -> &[Self::P; N];
    /// Returns current centroid positions.
    fn centroids(&self) -> &[Self::P; K];
    /// Returns per-point distance bounds.
    fn boundings(&self) -> &[Self::B; N];
    /// Initializes centroids (typically k-means++).
    fn init_centroids(&self) -> [Self::P; K];
    /// Initializes bounds by computing all point-centroid distances.
    fn init_bounds(&self) -> Box<[Self::B; N]> {
        (0..N)
            .into_par_iter()
            .map(|i| self.neighbor(i))
            .map(Self::B::from)
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
    fn centroid(&self, j: usize) -> &Self::P {
        &self.centroids()[j]
    }
    /// Gets bounds for point by index.
    fn bound(&self, i: usize) -> &Self::B {
        &self.boundings()[i]
    }
    /// Number of iterations to run.
    fn t(&self) -> usize {
        1024
    }
    /// Finds nearest centroid for a point (O(K) distance calls).
    fn neighbor(&self, i: usize) -> (usize, f32) {
        let x = self.point(i);
        self.centroids()
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
        if i == j { 0.0 } else { self.distance(self.centroid(i), self.centroid(j)) }
    }

    /// Computes s(c) = (1/2) min_{c'≠c} d(c, c') for each centroid.
    fn midpoints(&self, pairwise: &[[f32; K]; K]) -> [f32; K] {
        let mut result = [f32::MAX; K];
        pairwise.iter().enumerate().for_each(|(i, row)| {
            row.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .for_each(|(_, &d)| result[i] = result[i].min(d * 0.5));
        });
        result
    }

    /// Computes how far each centroid moved this iteration.
    fn drift(&self, news: &[Self::P]) -> Drift<K> {
        Drift::from_array(std::array::from_fn(|i| self.distance(&news[i], self.centroid(i))))
    }

    /// Computes new centroids from current assignments. Each new
    /// centroid is the result of absorbing every point assigned to
    /// that cluster into the running mean.
    fn recompute(&self, bounds: &[Self::B]) -> [Self::P; K] {
        use rayon::prelude::*;
        let centroids = (0..K)
            .into_par_iter()
            .map(|j| {
                bounds
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.j() == j)
                    .map(|(i, _)| self.point(i))
                    .fold(self.centroid(j).identity(), Self::P::absorb)
            })
            .collect::<Vec<_>>();
        std::array::from_fn(|j| centroids[j])
    }

    /// Assigns every point to its nearest current centroid, computing only the distances its
    /// bounds cannot rule out. The bounds must hold for the current centroids, as every
    /// `step_elkan` leaves them.
    fn assign(&self, bounds: &mut [Self::B; N]) {
        let pairwise = self.pairwises();
        let midpoints = self.midpoints(&pairwise);
        bounds
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, b)| b.settle(&pairwise, &midpoints, |j| self.distance(self.point(i), self.centroid(j))));
    }

    /// Executes one Elkan iteration with bound maintenance.
    ///
    /// Returns `(new_centroids, drift)` so the caller can report convergence telemetry without
    /// recomputing distances.
    fn step_elkan(&self, bounds: &mut [Self::B; N]) -> ([Self::P; K], Drift<K>) {
        self.assign(bounds);
        let centroids = self.recompute(bounds);
        let drift = self.drift(&centroids);
        bounds.par_iter_mut().for_each(|b| b.update(&drift));
        (centroids, drift)
    }

    /// Executes one naive iteration (for verification/benchmarking).
    fn step_naive(&self) -> [Self::P; K] {
        let identity = self.centroid(0).identity();
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

    /// Computes root-mean-square error against an explicit bounds buffer.
    /// Pulled out of `rms()` so callers that own the bounds (e.g. an
    /// in-progress iteration that has swapped it out of the layer) can
    /// still report convergence.
    fn rms_with(&self, bounds: &[Self::B; N]) -> Energy {
        (bounds
            .par_iter()
            .enumerate()
            .map(|(i, b)| self.distance(self.point(i), self.centroid(b.j())))
            .map(|d| d * d)
            .sum::<Energy>()
            / N as Energy)
            .sqrt()
    }

    /// Computes root-mean-square error using the layer's own bounds.
    /// Convenience wrapper around `rms_with(self.bounds())`.
    fn rms(&self) -> Energy {
        self.rms_with(self.boundings())
    }
}
