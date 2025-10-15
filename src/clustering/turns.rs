use super::*;
use crate::Energy;

/// it just so happens that we can cluster arbitrary
/// subsets of Turn Histogram equity distributions,
/// because we project into the River space on the fly (obs.equity())
#[derive(Clone)]
pub struct TurnLayer {
    metric: Metric,
    points: Vec<Histogram>,
    kmeans: Vec<Histogram>,
    bounds: Vec<Bound>,
}

impl TurnLayer {
    const fn n() -> usize {
        64
    }
    const fn k() -> usize {
        4
    }
    const fn t() -> usize {
        8
    }
    pub fn new() -> Self {
        use crate::cards::observation::Observation;
        use crate::cards::street::Street;
        use crate::clustering::histogram::Histogram;
        let points = (0..Self::n())
            .map(|_| Histogram::from(Observation::from(Street::Turn)))
            .collect::<Vec<_>>();
        let kmeans = Vec::default();
        let bounds = Vec::default();
        let metric = Metric::default();
        let mut km = Self {
            metric,
            points,
            kmeans,
            bounds,
        };
        km.kmeans = km.init_kmeans();
        km.bounds = km.init_bounds();
        km
    }
    pub fn step_elkan(&mut self) {
        let (kmeans, bounds) = self.next_eklan();
        self.kmeans = kmeans;
        self.bounds = bounds;
    }
    pub fn step_naive(&mut self) {
        let (kmeans, bounds) = self.next_naive();
        self.kmeans = kmeans;
        self.bounds = bounds;
    }
}

impl Elkan for TurnLayer {
    type P = Histogram;
    fn t(&self) -> usize {
        Self::t()
    }
    fn n(&self) -> usize {
        Self::n()
    }
    fn k(&self) -> usize {
        Self::k()
    }
    fn dataset(&self) -> &Vec<Histogram> {
        &self.points
    }
    fn kmeans(&self) -> &Vec<Histogram> {
        &self.kmeans
    }
    fn bounds(&self) -> &Vec<Bound> {
        &self.bounds
    }
    fn distance(&self, h1: &Histogram, h2: &Histogram) -> Energy {
        self.metric.emd(h1, h2)
    }
    fn init_kmeans(&self) -> Vec<Histogram> {
        use crate::cards::observation::Observation;
        use crate::cards::street::Street;
        let k = self.k();
        (0..k)
            .map(|_| Histogram::from(Observation::from(Street::Turn)))
            .collect()
    }
}
