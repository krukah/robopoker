use super::*;
use crate::Energy;
use crate::cards::*;

/// Test fixture for Elkan algorithm - clusters arbitrary Turn histograms.
/// Uses small fixed constants for fast testing.
const K: usize = 8;
const N: usize = 2048;

#[derive(Clone)]
pub struct TestLayer {
    metric: Metric,
    kmeans: [Histogram; K],
    points: Box<[Histogram; N]>,
    bounds: Box<[Bounds<K>; N]>,
}

impl TestLayer {
    const fn t() -> usize {
        8
    }
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
        km.kmeans = km.init_kmeans();
        km.bounds = km.init_bounds();
        km
    }

    pub fn step(&mut self) {
        let next = vec![Bounds::default(); N].try_into().expect("N");
        let ref mut curr = self.bounds;
        let ref mut prev = std::mem::replace(curr, next);
        self.kmeans = Elkan::step_elkan(self, prev);
        let ref mut curr = self.bounds;
        std::mem::swap(prev, curr);
        self.heal();
    }

    pub fn naive(&mut self) {
        self.kmeans = Elkan::step_naive(self);
        self.heal();
    }

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
    fn kmeans(&self) -> &[Histogram; K] {
        &self.kmeans
    }
    fn bounds(&self) -> &[Bounds<K>; N] {
        &self.bounds
    }
    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
        self.metric.emd(h1, h2)
    }
    fn init_kmeans(&self) -> [Histogram; K] {
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
