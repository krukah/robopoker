use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::consumer::Consumer;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric as _;
use crate::clustering::producer::Producer;
use crate::clustering::progress::Progress;
use crate::clustering::projection::Projection as _;
use crate::clustering::xor::Pair;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::Arc;

/// Centroid is a wrapper around two histograms.
/// We use it to swap the current and next histograms
/// after each iteration of kmeans clustering.
struct Centroid {
    curr: Histogram,
    next: Histogram,
}
impl Centroid {
    fn switch(&mut self) {
        self.curr.destroy();
        std::mem::swap(&mut self.curr, &mut self.next);
    }
    fn absorb(&mut self, h: &Histogram) {
        self.next.absorb(h);
    }
    fn reveal(&self) -> &Histogram {
        &self.curr
    }
}

/// this is an intermediate data structure
/// used during the clustering process.
/// we need to hold on to histograms immutable
/// while computing K means calculations.
/// using the outer layer's abstraction map,
/// we project Observation -> [Observation] -> [Abstraction] -> Histogram.
struct LargeSpace(BTreeMap<Observation, Histogram>);
impl LargeSpace {
    fn histogram(&self, o: &Observation) -> &Histogram {
        self.0.get(o).expect("observation projection")
    }
}

/// this is an intermediate data structure
/// used during the clustering process.
/// we need to hold on to histograms immutable
/// while computing K means calculations.
/// using the outer layer's abstraction map,
/// we project Observation -> [Observation] -> [Abstraction] -> Histogram.
struct SmallSpace(BTreeMap<Abstraction, Centroid>);
impl SmallSpace {
    fn histogram(&self, a: &Abstraction) -> &Histogram {
        self.0.get(a).expect("abstraction projection").reveal()
    }
}
/// this is the output of the clustering module
/// it is a massive table of Observation -> Abstraction.
/// effectively, this is a compressed representation of the
/// full game tree, learned by kmeans
/// rooted in showdown equity at the River.
struct Abstractor(BTreeMap<Observation, Abstraction>);
impl Abstractor {
    /// iterating over all Observations for a given Street,
    /// map each into a Histogram,
    /// and collect into a LargeSpace.
    fn generate(&self, street: Street) -> LargeSpace {
        LargeSpace(
            Observation::all(street)
                .into_iter()
                .map(|inner| (inner, self.assemble(&inner)))
                .collect::<BTreeMap<Observation, Histogram>>(),
        )
    }
    /// at a given Street,
    /// decompose the Observation,
    /// into all of its next-street Observations,
    /// and map each of them into an Abstraction,
    /// and collect the results into a Histogram.
    fn assemble(&self, inner: &Observation) -> Histogram {
        match inner.street() {
            Street::Turn => inner.clone().into(),
            _ => inner
                .outnodes()
                .into_iter()
                .map(|ref outer| self.represent(outer))
                .collect::<Vec<Abstraction>>()
                .into(),
        }
    }
    /// lookup the pre-computed abstraction for the outer observation
    fn represent(&self, outer: &Observation) -> Abstraction {
        self.0
            .get(outer)
            .cloned()
            .expect("precomputed abstraction mapping")
    }
}

/// Distance metric for kmeans clustering.
/// encapsulates distance between Abstractions of the "previous" hierarchy,
/// as well as: distance between Histograms of the "current" hierarchy.
struct Metric(BTreeMap<Pair, f32>);
impl Metric {
    fn distance(&self, a: &Abstraction, b: &Abstraction) -> f32 {
        self.0.distance(a, b)
    }
    fn wasserstein(&self, a: &Histogram, b: &Histogram) -> f32 {
        self.0.emd(a, b)
    }
}

// horizontal scaling across threads for k-means initialization and clustering
// observation_abstraction: BTreeMap<Observation, Abstraction>
// observation_distributio: BTreeMap<Observation, Histogram>
// abstraction_distributio: BTreeMap<Abstraction, Histogram>
//
// INITIALIZATION:
// each shard needs:
// - Arc<Vec<Histogram>>                        a readonly view of all N Histograms
// - Arc<Vec<Observation>>                      a readonly view of all N Observations
// - Fn(Observation) -> Histogram               Histogram from readonly Observation
// - Fn(Histogram, Histogram) -> Abstraction    Abstraction from two Histograms
//
// CLUSTERING:
// each shard needs:
// - Fn(Observation) -> Histogram               Histogram from Observation; self.projection
// - Fn(Abstraction) -> &mut Histogram          Histogram from nearest neighbor Abstraction; absorb()
// - Fn(Observation) -> &mut Abstraction        nearest neighbor Abstraction; assign()

/// KMeans hiearchical clustering. Every Observation is to be clustered with "similar" observations. River cards are the base case, where similarity metric is defined by equity. For each higher layer, we compare distributions of next-layer outcomes. Distances are measured by EMD and unsupervised kmeans clustering is used to cluster similar distributions. Potential-aware imperfect recall!
pub struct Layer {
    street: Street,
    metric: BTreeMap<Pair, f32>, // impl Metric
    points: BTreeMap<Observation, (Histogram, Abstraction)>, // impl Projection
    kmeans: BTreeMap<Abstraction, (Histogram, Histogram)>,
}

impl Layer {
    pub async fn hierarchical() -> Self {
        Self::outer()
            .inner()
            .await
            .upload()
            .inner()
            .await
            .upload()
            .inner()
            .await
            .upload()
    }
    /// async equity calculations to create initial River layer.
    pub fn outer() -> Self {
        Self {
            street: Street::Rive,
            points: BTreeMap::default(),
            kmeans: BTreeMap::default(),
            metric: BTreeMap::default(),
        }
    }

    /// Yield the next layer of abstraction by kmeans clustering. The recursive nature of layer methods encapsulates the hiearchy of learned abstractions via kmeans.
    /// TODO; make this async and persist to database after each layer
    pub async fn inner(mut self) -> Self {
        self.street = self.street.prev();
        self.points = self.projection().await;
        self.kmeans = self.initialize();
        self.cluster();
        self.metric = self.metric();
        self
    }

    /// projection is the async task of mapping observations
    /// to their nearest neighbor abstractions.
    /// for Street::Turn, we use equity calculations
    /// of following Street::River observations.
    /// for other Streets, we use previously calculated
    /// abstractions for the previous street's observations
    async fn projection(&self) -> BTreeMap<Observation, (Histogram, Abstraction)> {
        log::info!("projection {}", self.street);
        if self.street == Street::Turn {
            let ref observations = Arc::new(Observation::all(Street::Turn));
            let (tx, rx) = tokio::sync::mpsc::channel::<(Observation, Histogram)>(1024);
            let consumer = Consumer::new(rx);
            let consumer = tokio::spawn(consumer.run());
            let producers = (0..num_cpus::get())
                .map(|i| Producer::new(i, tx.clone(), observations.clone()))
                .map(|p| tokio::spawn(p.run()))
                .collect::<Vec<_>>();
            std::mem::drop(tx);
            futures::future::join_all(producers).await;
            consumer.await.expect("equity mapping task completes")
        } else {
            Observation::all(self.street)
                .into_iter()
                .map(|obs| (obs, (self.points.project(obs), Abstraction::random())))
                .collect()
        }
    }

    /// compute the metric of the next innermost layer.
    /// take outer product of centroid histograms over measure.
    /// we calculate this matrix only after the kmeans
    /// clustering abstraction is computed for this layer.
    /// we persist for use in the next layer.
    fn metric(&self) -> BTreeMap<Pair, f32> {
        log::info!("computing metric {}", self.street);
        let mut metric = BTreeMap::new();
        for (i, (x, _)) in self.kmeans.iter().enumerate() {
            for (j, (y, _)) in self.kmeans.iter().enumerate() {
                if i > j {
                    let index = Pair::from((x, y));
                    let ref x = self.kmeans.get(x).expect("in centroids").0; // Centroid::prev()
                    let ref y = self.kmeans.get(y).expect("in centroids").0; // Centroid::prev()
                    let distance = self.metric.emd(x, y) + self.metric.emd(y, x);
                    let distance = distance / 2.0;
                    metric.insert(index, distance);
                }
            }
        }
        metric
    }
}

/*
kmeans initialization
   1. choose first centroid randomly from the dataset
   2. choose nth centroid with probability proportional to squared distance of nearest neighbors
   3. collect histograms and label with arbitrary (random) Abstractions

kmeans clustering
   1. assign each observation to the nearest centroid
   2. update each centroid by averaging the observations assigned to it
   3. repeat for t iterations
*/

impl Layer {
    /// K Means++ implementation yields initial histograms
    /// Abstraction labels are random and require uniqueness.
    fn initialize(&self) -> BTreeMap<Abstraction, (Histogram, Histogram)> {
        log::info!("initializing kmeans {}", self.street);
        // 1. Choose 1st centroid randomly from the dataset
        let ref mut rng = rand::rngs::StdRng::seed_from_u64(self.street as u64);
        let mut kmeans = Vec::<Histogram>::new();
        let ref histograms = self
            .points
            .values()
            .map(|(hist, _)| hist)
            .collect::<Vec<&Histogram>>();
        let first = histograms
            .choose(rng)
            .cloned()
            .cloned()
            .expect("non-empty lower observations");
        kmeans.push(first);
        // 2. Choose nth centroid with probability proportional to squared distance of nearest neighbors
        let mut progress = Progress::new(self.k(), 10);
        while kmeans.len() < self.k() {
            let ref mut kmeans = kmeans;
            let weights = histograms
                .iter()
                .map(|histogram| self.proximity(histogram, kmeans))
                .map(|min| min * min)
                .collect::<Vec<f32>>();
            let choice = WeightedIndex::new(weights)
                .expect("valid weights array")
                .sample(rng);
            let sample = histograms
                .get(choice)
                .cloned()
                .cloned()
                .expect("shared index with outer layer");
            kmeans.push(sample);
            progress.tick();
        }
        // 3. Collect histograms and label with arbitrary (random) Abstractions
        kmeans
            .into_iter()
            .map(|mean| (Abstraction::random(), (mean, Histogram::default())))
            .collect()
    }

    /// Run kmeans iterations.
    /// Presumably, we have been generated by a previous layer, with the exception of Outer == River.
    /// After the base case, we trust that our observations, abstractions, and metric are correctly populated.
    fn cluster(&mut self) {
        assert!(self.kmeans.len() >= self.k());
        log::info!("kmeans clustering {}", self.street);
        let ref mut progress = Progress::new(self.t(), 100);
        for _ in 0..self.t() {
            for observation in self
                .points
                .keys()
                .copied()
                .into_iter()
                .collect::<Vec<_>>()
                .iter()
            {
                let ref neighbor = self.nearest(observation);
                self.assign(observation, neighbor);
                self.absorb(observation, neighbor);
            }
            self.recycle();
            progress.tick();
        }
    }

    /// Find the minimum distance between a histogram and
    /// a list of already existing centroids
    /// for k means ++ initialization
    fn proximity(&self, x: &Histogram, centroids: &Vec<Histogram>) -> f32 {
        centroids
            .iter()
            .map(|target| self.metric.emd(x, target))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("find nearest neighbor")
    }

    /// find the nearest neighbor for a given observation
    /// returns the node abstraction that is closest to the observation
    fn nearest(&self, observation: &Observation) -> Abstraction {
        let mut nearests = f32::MAX;
        let mut neighbor = Abstraction::random();
        let ref histogram = self.points.get(observation).expect("in continuations").0;
        for (centroid, (target, _)) in self.kmeans.iter() {
            let distance = self.metric.emd(histogram, target);
            if distance < nearests {
                nearests = distance;
                neighbor = centroid.to_owned();
            }
        }
        neighbor
    }

    /// assign the given observation to the specified neighbor
    /// by updating self.distributions mapping
    /// on each iteration, we update the abstraction of the observation
    fn assign(&mut self, observation: &Observation, neighbor: &Abstraction) {
        self.points
            .get_mut(observation)
            .expect("in continuations")
            .1 = neighbor.to_owned();
    }

    /// absorb the observation into the specified neighbor
    /// by updating self.kabstractions mapping
    /// we only update the .1 Histogram which is NOT used to calculate kmeans
    /// for everyone else on this iteration.
    /// they get swapped and cleared on the next iteration.
    fn absorb(&mut self, observation: &Observation, neighbor: &Abstraction) {
        let ref children = self.points.get(observation).expect("in continuations").0;
        self.kmeans
            .get_mut(neighbor)
            .expect("kabstractions was initialized with neighbor")
            .1
            .absorb(children);
        // Centroid::absorb
    }

    /// forget the old centroids and clear the new ones
    /// basically recylce memory between iterations
    /// out with the old and in with the new
    fn recycle(&mut self) {
        for (_, (old, new)) in self.kmeans.iter_mut() {
            old.destroy();
            std::mem::swap(old, new);
        }
    }

    /// Number of centroids in k means on inner layer. Loosely speaking, the size of our abstraction space.
    fn k(&self) -> usize {
        match self.street {
            Street::Turn => 200,
            Street::Flop => 200,
            Street::Pref => 169,
            _ => unreachable!("how did you get here"),
        }
    }

    /// Number of kmeans iterations to run on current layer.
    fn t(&self) -> usize {
        match self.street {
            Street::Turn => 100,
            Street::Flop => 100,
            Street::Pref => 10,
            _ => unreachable!("how did you get here"),
        }
    }
}

/*
persistence methods
*/
const BUFFER: usize = 1024 * 1024 * 1024;
impl Layer {
    /// Write to file. We'll open a new file for each layer, whatever.
    pub fn upload(self) -> Self {
        self.truncate();
        self.upload_distance();
        self.upload_centroid();
        self
    }

    /// Truncate the files
    fn truncate(&self) {
        std::fs::remove_file(format!("centroid_{}.bin", self.street)).ok();
        std::fs::remove_file(format!("distance_{}.bin", self.street)).ok();
    }

    /// Write centroid data to a file
    fn upload_centroid(&self) {
        log::info!("uploading centroids {}", self.street);
        let mut file =
            std::fs::File::create(format!("centroid_{}.bin", self.street)).expect("create file");
        let mut progress = Progress::new(self.points.len(), 10);
        for (observation, (_, abstraction)) in self.points.iter() {
            use std::io::Write;
            let obs = i64::from(*observation) as u64;
            let abs = i64::from(*abstraction) as u64;
            let ref bytes = [obs.to_le_bytes(), abs.to_le_bytes()].concat();
            file.write_all(bytes).expect("write to file");
            progress.tick();
        }
    }

    /// Write distance data to a file
    fn upload_distance(&self) {
        log::info!("uploading distance {}", self.street);
        let mut file =
            std::fs::File::create(format!("distance_{}.bin", self.street)).expect("create file");
        let mut progress = Progress::new(self.metric.len(), 10);
        for (pair, distance) in self.metric.iter() {
            use std::io::Write;
            let pair = i64::from(*pair) as u64;
            let distance = f64::from(*distance);
            let ref bytes = [pair.to_le_bytes(), distance.to_le_bytes()].concat();
            file.write_all(bytes).expect("write to file");
            progress.tick();
        }
    }

    /// read centroid data from a file
    pub fn download_centroid(street: Street) -> BTreeMap<Observation, Abstraction> {
        let mut map = BTreeMap::new();
        let file = std::fs::File::open(format!("centroid_{}.bin", street)).expect("file open");
        let ref mut reader = std::io::BufReader::with_capacity(BUFFER, file);
        let ref mut buffer = [0u8; 16];
        while reader.read_exact(buffer).is_ok() {
            let obs_u64 = u64::from_le_bytes(buffer[00..08].try_into().unwrap());
            let abs_u64 = u64::from_le_bytes(buffer[08..16].try_into().unwrap());
            let observation = Observation::from(obs_u64 as i64);
            let abstraction = Abstraction::from(abs_u64 as i64);
            map.insert(observation, abstraction);
        }
        map
    }

    /// read distance data from a file
    pub fn download_distance(street: Street) -> BTreeMap<Pair, f32> {
        let mut map = BTreeMap::new();
        let file = std::fs::File::open(format!("distance_{}.bin", street)).expect("file open");
        let ref mut reader = std::io::BufReader::with_capacity(BUFFER, file);
        let ref mut buffer = [0u8; 12];
        while reader.read_exact(buffer).is_ok() {
            let pair_u64 = u64::from_le_bytes(buffer[00..08].try_into().unwrap());
            let dist_f64 = f64::from_le_bytes(buffer[08..16].try_into().unwrap());
            let pair = Pair::from(pair_u64 as i64);
            let distance = dist_f64 as f32;
            map.insert(pair, distance);
        }
        map
    }
}
