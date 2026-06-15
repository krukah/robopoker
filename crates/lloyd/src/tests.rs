use super::*;
use deuce::*;
use pokerkit::Energy;

/// Test fixture for Elkan algorithm verification.
///
/// Clusters random Turn histograms with small fixed constants for
/// fast unit testing. Verifies that Elkan produces identical results
/// to naive k-means while demonstrating convergence properties.
const K: usize = 8;
const N: usize = 2048;

/// Test layer implementing Elkan trait for algorithm verification.
#[derive(Clone)]
pub struct TestLayer {
    metric: Metric,
    kmeans: [Histogram; K],
    points: Box<[Histogram; N]>,
    bounds: Box<[Bounds<K>; N]>,
}

impl TestLayer {
    /// Number of iterations for test runs.
    const fn t() -> usize {
        8
    }
    /// Creates a new test layer with random Turn histograms.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let points = (0..N)
            .map(|_| Histogram::from(Observation::from(Street::Turn)))
            .collect::<Vec<_>>()
            .try_into()
            .expect("N");
        let metric = Metric::default();
        let mut km = Self {
            metric,
            points,
            kmeans: std::array::from_fn(|_| Histogram::empty(Street::Rive)),
            bounds: vec![Bounds::default(); N].try_into().expect("N"),
        };
        km.kmeans = km.init_centroids();
        km.bounds = km.init_bounds();
        km
    }

    /// Runs one Elkan iteration.
    pub fn step(&mut self) {
        let next = vec![Bounds::default(); N].try_into().expect("N");
        let ref mut curr = self.bounds;
        let ref mut prev = std::mem::replace(curr, next);
        let (kmeans, _drifts) = Elkan::step_elkan(self, prev);
        self.kmeans = kmeans;
        let ref mut curr = self.bounds;
        std::mem::swap(prev, curr);
        self.heal();
    }

    /// Runs one naive iteration.
    pub fn naive(&mut self) {
        self.kmeans = Elkan::step_naive(self);
        self.heal();
    }

    /// Replaces empty clusters with random histograms.
    pub fn heal(&mut self) {
        self.kmeans
            .iter_mut()
            .filter(|h| h.n() == 0)
            .map(|h| *h = Histogram::from(Observation::from(Street::Turn)))
            .count();
    }
}

impl Elkan<K, N> for TestLayer {
    type P = Histogram;

    fn t(&self) -> usize {
        Self::t()
    }

    fn points(&self) -> &[Histogram; N] {
        &self.points
    }

    fn centroids(&self) -> &[Histogram; K] {
        &self.kmeans
    }

    fn boundings(&self) -> &[Bounds<K>; N] {
        &self.bounds
    }

    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
        self.metric.emd(h1, h2)
    }

    fn init_centroids(&self) -> [Histogram; K] {
        std::array::from_fn(|_| Histogram::from(Observation::from(Street::Turn)))
    }

    fn rms(&self) -> Energy {
        use rayon::prelude::*;
        ((0..N)
            .into_par_iter()
            .map(|i| self.neighbor(i).1)
            .map(|d| d * d)
            .sum::<Energy>()
            / N as Energy)
            .sqrt()
    }
}

#[cfg(test)]
mod verify {
    use super::TestLayer;
    use elkan::Elkan;

    #[test]
    #[ignore = "flaky: stochastic kmeans"]
    /// RMS should be non-increasing across Elkan iterations (modulo empty-cluster heal).
    fn elkan_rms_decreases() {
        let mut km = TestLayer::new();
        let mut rms = vec![km.rms()];
        for _ in 0..km.t() {
            km.step();
            rms.push(km.rms());
        }
        for window in rms.windows(2) {
            assert!(window[0] >= window[1], "RMS increasing: {} -> {}", window[0], window[1]);
        }
    }

    #[test]
    #[ignore = "flaky: stochastic kmeans"]
    /// RMS should be roughly converged after t() iterations.
    fn elkan_rms_converges() {
        let mut km = TestLayer::new();
        for _ in 0..km.t() {
            km.step();
        }
        let r1 = km.rms();
        km.step();
        let r2 = km.rms();
        assert!((r1 - r2).abs() <= 0.01, "RMS is unlikely large: {r1} -> {r2}");
    }

    #[test]
    #[ignore = "slow: full naive comparison"]
    /// Elkan must be bit-identical to naive k-means at every iteration.
    fn elkan_naive_equivalence() {
        let km = TestLayer::new();
        let mut elkan = km.clone();
        let mut naive = km.clone();
        for _ in 0..km.t() {
            elkan.step();
            naive.naive();
            assert_eq!(elkan.rms(), naive.rms());
            assert_eq!(elkan.centroids(), naive.centroids());
        }
    }
}
