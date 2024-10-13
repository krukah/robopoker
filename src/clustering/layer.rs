use super::abstractor::Abstractor;
use super::datasets::AbstractionSpace;
use super::datasets::ObservationSpace;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric;
use crate::clustering::xor::Pair;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::collections::BTreeMap;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Hierarchical K Means Learner
/// this is decomposed into the necessary data structures
/// for kmeans clustering to occur for a given `Street`.
/// it should also parallelize well, with kmeans and lookup
/// being the only mutable fields.
pub struct Layer {
    street: Street,
    metric: Metric,
    lookup: Abstractor,
    kmeans: AbstractionSpace,
    points: ObservationSpace,
}

impl Layer {
    /// from scratch, generate and persist the full Abstraction lookup table
    pub fn learn() {
        Self::outer()
            .inner() // turn
            .save()
            .inner() // flop
            .save();
        todo!("add the abstraction-less PreFlop Observations"); // TODO
                                                                // add the abstraction-less PreFlop Observations
                                                                // or include a Abstraction::PreFlop(Hole) variant
                                                                // to make sure we cover the full set of Observations
    }

    /// start with the River layer. everything is empty because we
    /// can generate `Abstractor` and `SmallSpace` from "scratch".
    /// - `lookup`: lazy equity calculation of river observations
    /// - `kmeans`: equity percentile buckets of equivalent river observations
    /// - `metric`: absolute value of `Abstraction::Equity` difference
    /// - `points`: not used for inward projection. only used for clustering. and no clustering on River.
    fn outer() -> Self {
        Self {
            lookup: Abstractor::default(),
            kmeans: AbstractionSpace::default(),
            points: ObservationSpace::default(),
            metric: Metric::default(),
            street: Street::Rive,
        }
    }
    /// hierarchically, recursively generate the inner layer
    fn inner(&self) -> Self {
        let mut layer = Self {
            lookup: Abstractor::default(),       // assigned during clustering
            kmeans: AbstractionSpace::default(), // assigned during clustering
            street: self.inner_street(),         // uniquely determined by outer layer
            metric: self.inner_metric(),         // uniquely determined by outer layer
            points: self.inner_points(),         // uniquely determined by outer layer
        };
        layer.initial();
        layer.cluster();
        layer
    }

    /// simply go to the previous street
    fn inner_street(&self) -> Street {
        log::info!("advancing from {} to {}", self.street, self.street.prev());
        self.street.prev()
    }
    /// compute the outer product of the `Abstraction -> Histogram`s at the current layer,
    /// - generate the _inner layer_ `Metric` between `Abstraction`s
    /// - by using the _outer layer_ `Metric` between `Histogram`s via EMD calcluations.
    ///
    /// we symmetrize the distance by averaging the EMDs in both directions.
    /// the distnace isn't symmetric in the first place only because our heuristic algo is not fully accurate
    fn inner_metric(&self) -> Metric {
        log::info!("computing metric {}", self.street);
        let mut metric = BTreeMap::new();
        for a in self.kmeans.0.keys() {
            for b in self.kmeans.0.keys() {
                if a > b {
                    let index = Pair::from((a, b));
                    let x = self.kmeans.0.get(a).expect("pre-computed").reveal();
                    let y = self.kmeans.0.get(b).expect("pre-computed").reveal();
                    let distance = self.metric.emd(x, y) + self.metric.emd(y, x);
                    let distance = distance / 2.0;
                    metric.insert(index, distance);
                }
            }
        }
        Metric(metric)
    }

    /// using the current layer's `Abstractor`,
    /// we generate the `LargeSpace` of `Observation` -> `Histogram`.
    /// 1. take all `Observation`s for `self.street.prev()`
    /// 2. map each to possible `self.street` `Observation`s
    /// 3. use `self.abstractor` to map each into an `Abstraction`
    /// 4. collect `Abstraction`s into a `Histogram`, for each `Observation`
    fn inner_points(&self) -> ObservationSpace {
        log::info!("computing projections {}", self.street);
        let exhausted = Observation::exhaust(self.street.prev());

        let pb = ProgressBar::new(exhausted.len() as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {spinner:.green} {wide_bar} ETA {eta_precise}",
            )
            .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(100));

        ObservationSpace(
            exhausted
                .into_par_iter()
                .map(|inner| (inner, self.lookup.projection(&inner)))
                .inspect(|_| pb.inc(1))
                .collect::<BTreeMap<Observation, Histogram>>(),
        )
    }

    /// initializes the centroids for k-means clustering using the k-means++ algorithm
    /// 1. choose 1st centroid randomly from the dataset
    /// 2. choose nth centroid with probability proportional to squared distance of nearest neighbors
    /// 3. collect histograms and label with arbitrary (random) `Abstraction`s
    ///
    /// if this becomes a bottleneck with contention,
    /// consider partitioning dataset or using lock-free data structures.
    fn initial(&mut self) {
        log::info!("initializing kmeans {}", self.street);
        let ref mut rng = rand::rngs::StdRng::seed_from_u64(self.street as u64);
        let histogram = self.sample_uniform(rng);
        self.kmeans.expand(histogram);
        while self.k() > self.l() {
            log::info!("add initial {}", self.l());
            let histogram = self.sample_outlier(rng);
            self.kmeans.expand(histogram);
        }
    }
    /// for however many iterations we want,
    /// 1. assign each `Observation` to the nearest `Centroid`
    /// 2. update each `Centroid` by averaging the `Observation`s assigned to it
    ///
    /// if this becomes a bottleneck with contention,
    /// consider partitioning dataset or using lock-free data structures.
    fn cluster(&mut self) {
        log::info!("clustering kmeans {}", self.street);
        for i in 0..self.t() {
            log::info!("computing abstractions {} {}", self.street, i);
            let abstractions = self
                .points
                .0
                .par_iter()
                .map(|(_, h)| self.nearest_neighbor(h))
                .collect::<Vec<Abstraction>>();
            log::info!("assigning abstractions {} {}", self.street, i);
            for ((o, h), a) in self.points.0.iter_mut().zip(abstractions.iter()) {
                self.lookup.assign(a, o);
                self.kmeans.absorb(a, h);
            }
            log::info!("resetting abstractions {} {}", self.street, i);
            for (_, centroid) in self.kmeans.0.iter_mut() {
                centroid.rotate();
            }
        }
    }

    /// the first Centroid is uniformly random across all `Observation` `Histogram`s
    fn sample_uniform(&self, rng: &mut StdRng) -> Histogram {
        self.points
            .0
            .values()
            .choose(rng)
            .expect("observation projections have been populated")
            .clone()
    }
    /// each next Centroid is selected with probability proportional to
    /// the squared distance to the nearest neighboring Centroid.
    /// faster convergence, i guess. on the shoulders of giants
    fn sample_outlier(&self, rng: &mut StdRng) -> Histogram {
        let weights = self
            .points
            .0
            .par_iter()
            .map(|(_, hist)| self.nearest_distance(hist))
            .collect::<Vec<f32>>();
        let index = WeightedIndex::new(weights)
            .expect("valid weights array")
            .sample(rng);
        self.points
            .0
            .values()
            .nth(index)
            .expect("shared index with outer layer")
            .clone()
    }

    /// distance^2 to the nearest neighboring Centroid, for kmeans++ sampling
    fn nearest_distance(&self, histogram: &Histogram) -> f32 {
        self.kmeans
            .0
            .par_iter()
            .map(|(_, centroid)| centroid.reveal())
            .map(|centroid| self.metric.emd(histogram, centroid))
            .map(|min| min * min)
            .min_by(|dx, dy| dx.partial_cmp(dy).unwrap())
            .expect("find nearest neighbor")
    }
    /// find the nearest neighbor `Abstraction` to a given `Histogram` for kmeans clustering
    fn nearest_neighbor(&self, histogram: &Histogram) -> Abstraction {
        self.kmeans
            .0
            .par_iter()
            .map(|(abs, centroid)| (abs, centroid.reveal()))
            .map(|(abs, centroid)| (abs, self.metric.emd(histogram, centroid)))
            .min_by(|(_, dx), (_, dy)| dx.partial_cmp(dy).unwrap())
            .expect("find nearest neighbor")
            .0
            .clone()
    }

    /// hyperparameter: how many centroids to learn
    fn k(&self) -> usize {
        match self.street {
            Street::Turn => 128,
            Street::Flop => 128,
            _ => unreachable!("how did you get here"),
        }
    }
    /// hyperparameter: how many iterations to run kmeans
    fn t(&self) -> usize {
        match self.street {
            _ => 100,
        }
    }
    /// length of current kmeans centroids
    fn l(&self) -> usize {
        self.kmeans.0.len()
    }

    /// save the current layer's `Metric` and `Abstractor` to disk
    fn save(self) -> Self {
        let path = format!("{}.abstraction.pgcopy", self.street);
        self.metric.save(path.clone());
        self.lookup.save(path.clone());
        self
    }
}
