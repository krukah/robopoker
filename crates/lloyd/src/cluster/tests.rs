use super::*;
use deuce::*;
use pokerkit::Energy;

/// Small fixed sizes so Elkan verification runs fast.
const K: usize = 8;
const N: usize = 2048;

/// Test layer over random Turn histograms, used to check that Elkan matches
/// naive k-means and converges.
#[derive(Clone)]
pub struct TestLayer<B = Bounds<K>> {
    metric: Metric,
    kmeans: [Histogram; K],
    points: Box<[Histogram; N]>,
    bounds: Box<[B; N]>,
}

impl TestLayer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let points = (0..N)
            .map(|_| Histogram::from(Observation::from(Street::Turn)))
            .collect::<Vec<_>>()
            .try_into()
            .expect("N");
        let mut km = Self {
            metric: Metric::default(),
            points,
            kmeans: std::array::from_fn(|_| Histogram::empty(Street::Rive)),
            bounds: vec![Bounds::default(); N].try_into().expect("N"),
        };
        *km.centroids_mut() = km.init_centroids();
        *km.boundings_mut() = km.init_bounds();
        km
    }
}

impl<B> TestLayer<B>
where
    B: Prune<K>,
{
    /// Number of iterations for test runs.
    const fn t() -> usize {
        8
    }

    /// The same points and centroids, with bounds of another scheme built from scratch.
    pub fn recast<C>(&self) -> TestLayer<C>
    where
        C: Prune<K>,
    {
        let mut km = TestLayer {
            metric: self.metric(),
            points: self.points().to_vec().try_into().expect("N"),
            kmeans: *self.centroids(),
            bounds: vec![C::default(); N].try_into().expect("N"),
        };
        *km.boundings_mut() = km.init_bounds();
        km
    }

    /// Runs one Elkan iteration.
    pub fn step(&mut self) {
        let mut bounds = std::mem::replace(self.boundings_mut(), vec![B::default(); N].try_into().expect("N"));
        *self.centroids_mut() = Elkan::step_elkan(self, &mut bounds).0;
        *self.boundings_mut() = bounds;
        self.heal();
    }

    /// Runs one naive iteration.
    pub fn naive(&mut self) {
        *self.centroids_mut() = Elkan::step_naive(self);
        self.heal();
    }

    /// Replaces empty clusters with random histograms.
    pub fn heal(&mut self) {
        self.centroids_mut()
            .iter_mut()
            .filter(|h| h.n() == 0)
            .for_each(|h| *h = Histogram::from(Observation::from(Street::Turn)));
    }

    fn metric(&self) -> Metric {
        self.metric
    }

    fn centroids_mut(&mut self) -> &mut [Histogram; K] {
        &mut self.kmeans
    }

    fn boundings_mut(&mut self) -> &mut Box<[B; N]> {
        &mut self.bounds
    }
}

impl<B> Elkan<K, N> for TestLayer<B>
where
    B: Prune<K>,
{
    type P = Histogram;
    type B = B;

    fn t(&self) -> usize {
        Self::t()
    }

    fn points(&self) -> &[Histogram; N] {
        &self.points
    }

    fn centroids(&self) -> &[Histogram; K] {
        &self.kmeans
    }

    fn boundings(&self) -> &[B; N] {
        &self.bounds
    }

    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
        self.metric().emd(h1, h2)
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
    use super::*;

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
    /// After the last step, the bounds-pruned assignment lands every point on a nearest centroid.
    fn elkan_assign_matches_neighbor() {
        let mut km = TestLayer::new();
        let mut bounds = km.boundings().clone();
        for _ in 0..km.t() {
            *km.centroids_mut() = Elkan::step_elkan(&km, &mut bounds).0;
        }
        km.assign(&mut bounds);
        bounds
            .iter()
            .enumerate()
            .for_each(|(i, b)| assert_eq!(km.distance(km.centroid(b.j()), km.point(i)), km.neighbor(i).1));
    }

    #[test]
    /// Hamerly's pruned assignment also lands every point on a nearest centroid.
    fn hamerly_assign_matches_neighbor() {
        let mut km = TestLayer::new().recast::<Hamerly<K>>();
        let mut bounds = *km.boundings();
        for _ in 0..km.t() {
            *km.centroids_mut() = Elkan::step_elkan(&km, &mut bounds).0;
        }
        km.assign(&mut bounds);
        bounds
            .iter()
            .enumerate()
            .for_each(|(i, b)| assert_eq!(km.distance(km.centroid(b.j()), km.point(i)), km.neighbor(i).1));
    }

    #[test]
    /// Both bounds schemes prune only what cannot change, so they walk the same centroids.
    fn hamerly_elkan_equivalence() {
        let mut elkan = TestLayer::new();
        let mut hamerly = elkan.recast::<Hamerly<K>>();
        let mut e = elkan.boundings().clone();
        let mut h = *hamerly.boundings();
        for _ in 0..elkan.t() {
            *elkan.centroids_mut() = Elkan::step_elkan(&elkan, &mut e).0;
            *hamerly.centroids_mut() = Elkan::step_elkan(&hamerly, &mut h).0;
            assert_eq!(elkan.centroids(), hamerly.centroids());
        }
    }

    #[test]
    /// A `Kmeans` run hands the layer its bounds back, settled on nearest centroids.
    fn kmeans_returns_settled_bounds() {
        let mut layer = Layer::<K, N, Hamerly<K>>::random();
        **layer.centroids_mut() = layer.init_centroids();
        *layer.boundings_mut() = layer.init_bounds();
        let mut iter = layer.kmeans().with_bound(4);
        iter.by_ref().count();
        iter.settle();
        drop(iter);
        (0..N).for_each(|i| {
            assert_eq!(layer.distance(layer.centroid(layer.bound(i).j()), layer.point(i)), layer.neighbor(i).1);
        });
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
