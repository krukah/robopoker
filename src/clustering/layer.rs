use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::consumer::Consumer;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric;
use crate::clustering::producer::Producer;
use crate::clustering::progress::Progress;
use crate::clustering::projection::Projection;
use crate::clustering::xor::Pair;
use log::info;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::Arc;

/// KMeans hiearchical clustering. Every Observation is to be clustered with "similar" observations. River cards are the base case, where similarity metric is defined by equity. For each higher layer, we compare distributions of next-layer outcomes. Distances are measured by EMD and unsupervised kmeans clustering is used to cluster similar distributions. Potential-aware imperfect recall!
pub struct Layer {
    street: Street,
    metric: BTreeMap<Pair, f32>, // impl Metric
    points: BTreeMap<Observation, (Histogram, Abstraction)>, // impl Projection
    kmeans: BTreeMap<Abstraction, (Histogram, Histogram)>,
}

impl Layer {
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
        info!("projection {}", self.street);
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
        info!("computing metric {}", self.street);
        let mut metric = BTreeMap::new();
        for (i, (x, _)) in self.kmeans.iter().enumerate() {
            for (j, (y, _)) in self.kmeans.iter().enumerate() {
                if i > j {
                    let index = Pair::from((x, y));
                    let ref x = self.kmeans.get(x).expect("in centroids").0;
                    let ref y = self.kmeans.get(y).expect("in centroids").0;
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
        info!("initializing kmeans {}", self.street);
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
                .expect("shared index with lowers");
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
        info!("kmeans clustering {}", self.street);
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
        match self.street.prev() {
            Street::Turn => 200,
            Street::Flop => 200,
            Street::Pref => 169,
            _ => unreachable!("no other prev"),
        }
    }

    /// Number of kmeans iterations to run on current layer.
    fn t(&self) -> usize {
        match self.street.prev() {
            Street::Turn => 1_000,
            Street::Flop => 1_000,
            Street::Pref => 10,
            _ => unreachable!("no other prev"),
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
        info!("uploading centroids {}", self.street);
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
        info!("uploading distance {}", self.street);
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
            let obs_u64 = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
            let abs_u64 = u64::from_le_bytes(buffer[8..16].try_into().unwrap());
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
            let pair_u64 = u64::from_le_bytes(buffer[0..08].try_into().unwrap());
            let dist_f64 = f64::from_le_bytes(buffer[8..16].try_into().unwrap());
            let pair = Pair::from(pair_u64 as i64);
            let distance = dist_f64 as f32;
            map.insert(pair, distance);
        }
        map
    }
}
