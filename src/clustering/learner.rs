use super::abstractor::Abstractor;
use super::datasets::LargeSpace;
use super::datasets::SmallSpace;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric;
use crate::clustering::progress::Progress;
use crate::clustering::xor::Pair;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::io::Read;

/// Hierarchical K Means Learner
/// this is decomposed into the necessary data structures
/// for kmeans clustering to occur for a given `Street`.
/// it should also parallelize well, with kmeans and lookup
/// being the only mutable fields.
pub struct Hierarchical {
    street: Street,
    metric: Metric,
    points: LargeSpace,
    kmeans: SmallSpace,
    lookup: Abstractor,
}

impl Hierarchical {
    /// from scratch, generate and persist the full Abstraction lookup table
    pub fn cluster() {
        Self::outer()
            .inner() // turn
            .save()
            .inner() // flop
            .save();
    }
    /// if we have this full thing created we can also just retrieve it
    pub fn retrieve() -> Abstractor {
        let mut map = BTreeMap::default();
        map.extend(Self::load(Street::Turn).0);
        map.extend(Self::load(Street::Flop).0);
        Abstractor(map)
    }

    // layer generation

    /// start with the River layer. everything is empty because we
    /// can generate `Abstractor` and `SmallSpace` from "scratch".
    /// - `lookup`: lazy equity calculation of river observations
    /// - `kmeans`: equity percentile buckets of equivalent river observations
    /// - `metric`: absolute value of `Abstraction::Equity` difference
    /// - `points`: not used for inward projection. only used for clustering. and no clustering on River.
    fn outer() -> Self {
        Self {
            lookup: Abstractor::default(),
            kmeans: SmallSpace::default(),
            points: LargeSpace::default(),
            metric: Metric::default(),
            street: Street::Rive,
        }
    }
    /// hierarchically, recursively generate the inner layer
    fn inner(&self) -> Self {
        let mut inner = Self {
            lookup: Abstractor::default(), // assigned during clustering
            kmeans: SmallSpace::default(), // assigned during clustering
            street: self.inner_street(),   // uniquely determined by outer layer
            metric: self.inner_metric(),   // uniquely determined by outer layer
            points: self.inner_points(),   // uniquely determined by outer layer
        };
        inner.kmeans_initial();
        inner.kmeans_iterate();
        inner
    }

    // non-mutating layer generation

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
        for (i, x) in self.kmeans.0.keys().enumerate() {
            for (j, y) in self.kmeans.0.keys().enumerate() {
                if i > j {
                    let index = Pair::from((x, y));
                    let x = self.kmeans.0.get(x).expect("pre-computed").reveal();
                    let y = self.kmeans.0.get(y).expect("pre-computed").reveal();
                    let distance = self.metric.wasserstein(x, y) + self.metric.wasserstein(y, x);
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
    fn inner_points(&self) -> LargeSpace {
        log::info!("computing projections {}", self.street);
        use rayon::iter::IntoParallelIterator;
        use rayon::iter::ParallelIterator;
        LargeSpace(
            Observation::all(self.street.prev())
                .into_par_iter()
                .map(|inner| (inner, self.lookup.projection(&inner)))
                .collect::<BTreeMap<Observation, Histogram>>(),
        )
    }

    // k means clustering

    /// initializes the centroids for k-means clustering using the k-means++ algorithm
    /// 1. choose 1st centroid randomly from the dataset
    /// 2. choose nth centroid with probability proportional to squared distance of nearest neighbors
    /// 3. collect histograms and label with arbitrary (random) `Abstraction`s
    ///
    /// if this becomes a bottleneck with contention,
    /// consider partitioning dataset or using lock-free data structures.
    fn kmeans_initial(&mut self) {
        log::info!("initializing kmeans {}", self.street);
        let ref mut rng = rand::rngs::StdRng::seed_from_u64(self.street as u64);
        self.kmeans.extend(self.sample_uniform(rng));
        while self.kmeans.0.len() < self.k() {
            log::info!("+ {:3} of {:3}", self.kmeans.0.len(), self.k());
            self.kmeans.extend(self.sample_outlier(rng));
        }
    }
    /// for however many iterations we want,
    /// 1. assign each `Observation` to the nearest `Centroid`
    /// 2. update each `Centroid` by averaging the `Observation`s assigned to it
    ///
    /// if this becomes a bottleneck with contention,
    /// consider partitioning dataset or using lock-free data structures.
    fn kmeans_iterate(&mut self) {
        log::info!("iterating kmeans {}", self.street);
        for _ in 0..self.t() {
            for (o, h) in self.points.0.iter() {
                log::info!("assigning {}", o);
                let ref abstraction = self.nearest(h).clone(); // read self.metric, self.kmeans
                self.lookup.assign(abstraction, o); // write self.lookup. Observation won't collide
                self.kmeans.absorb(abstraction, h); // write self.kmeans. Abstraction could collide
            }
            for (_, centroid) in self.kmeans.0.iter_mut() {
                centroid.rotate();
            }
        }
    }
    /// find the nearest neighbor `Abstraction` to a given `Histogram`
    /// this might be expensive and worth benchmarking or profiling
    fn nearest(&self, histogram: &Histogram) -> &Abstraction {
        self.kmeans
            .0
            .iter()
            .map(|(abs, centroid)| (abs, self.metric.wasserstein(histogram, centroid.reveal())))
            .min_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
            .map(|(abs, _)| abs)
            .expect("find nearest neighbor")
    }

    // k means neighborhood + initialization

    /// the first point selected for initialization
    /// is uniformly random across all `Observation` `Histogram`s
    fn sample_uniform(&self, rng: &mut rand::rngs::StdRng) -> Histogram {
        self.points
            .0
            .values()
            .choose(rng)
            .expect("observation projections have been populated")
            .to_owned()
    }
    /// each next point is selected with probability proportional to
    /// the squared distance to the nearest neighboring centroid.
    /// faster convergence, i guess. on the shoulders of giants
    fn sample_outlier(&self, rng: &mut rand::rngs::StdRng) -> Histogram {
        let weights = self
            .points
            .0
            .values()
            .map(|hist| self.sample_weights(hist))
            .collect::<Vec<f32>>();
        let index = WeightedIndex::new(weights)
            .expect("valid weights array")
            .sample(rng);
        self.points
            .0
            .values()
            .nth(index)
            .expect("shared index with outer layer")
            .to_owned()
    }
    /// during K-means++ initialization, we sample any of
    /// the BigN `Observation`s with probability proportional to
    /// the squared distance to the nearest neighboring centroid.
    /// faster convergence, i guess. on the shoulders of giants
    fn sample_weights(&self, histogram: &Histogram) -> f32 {
        self.kmeans
            .0
            .values()
            .map(|centroid| centroid.reveal())
            .map(|mean| self.metric.wasserstein(histogram, mean))
            .map(|min| min * min)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("find nearest neighbor")
    }

    // hyperparametrization

    /// hyperparameter: how many centroids to learn
    fn k(&self) -> usize {
        match self.street {
            Street::Turn => 200,
            Street::Flop => 200,
            _ => unreachable!("how did you get here"),
        }
    }
    /// hyperparameter: how many iterations to run kmeans
    fn t(&self) -> usize {
        100
    }

    // file IO and persistence

    /// write the full abstraction lookup table to disk
    fn save(self) -> Self {
        log::info!("uploading abstraction lookup table {}", self.street);
        let mut file = std::fs::File::create(format!("{}", self.street)).expect("new file");
        let mut progress = Progress::new(self.lookup.0.len(), 10);
        for (observation, abstraction) in self.lookup.0.iter() {
            use std::io::Write;
            let obs = i64::from(*observation) as u64;
            let abs = i64::from(*abstraction) as u64;
            let ref bytes = [obs.to_le_bytes(), abs.to_le_bytes()].concat();
            file.write_all(bytes).expect("write to file");
            progress.tick();
        }
        self
    }
    /// read the full abstraction lookup table from disk
    fn load(street: Street) -> Abstractor {
        const BUFFER: usize = 1 << 16;
        log::info!("downloading abstraction lookup table {}", street);
        let mut map = BTreeMap::new();
        let file = std::fs::File::open(format!("{}", street)).expect("open file");
        let ref mut reader = std::io::BufReader::with_capacity(BUFFER, file);
        let ref mut buffer = [0u8; 16];
        while reader.read_exact(buffer).is_ok() {
            let obs_u64 = u64::from_le_bytes(buffer[00..08].try_into().unwrap());
            let abs_u64 = u64::from_le_bytes(buffer[08..16].try_into().unwrap());
            let observation = Observation::from(obs_u64 as i64);
            let abstraction = Abstraction::from(abs_u64 as i64);
            map.insert(observation, abstraction);
        }
        Abstractor(map)
    }
}
