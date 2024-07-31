// use tokio::sync::Mutex;

use super::abstraction::Abstraction;
use super::histogram::Centroid;
use super::histogram::Histogram;
use super::observation::Observation;
use super::persistence::storage::Storage;
use super::xor::Pair;
use crate::cards::street::Street;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use std::vec;
use tokio::sync::Mutex;

type Lookup = super::persistence::postgres::PostgresLookup;

pub struct Abstractor {
    storage: Lookup,
}

impl Abstractor {
    pub async fn new() -> Self {
        Self {
            storage: Lookup::new().await,
        }
    }

    async fn initials(&self) -> Vec<Centroid> {
        todo!("implement k-means++ initialization")
    }

    pub async fn river(&mut self) -> &mut Self {
        const TASKS: usize = 2;
        const RIVERS: usize = 2_809_475_760;
        const RIVRS_BATCH: usize = 16_384;
        const BATCHS_TASK: usize = RIVERS / TASKS / RIVRS_BATCH;
        let mut tasks = Vec::with_capacity(TASKS);
        let ref riversfull = Arc::new(Observation::all(Street::Rive));
        let ref mut progress = Arc::new(Mutex::new(Progress::new()));
        for itask in 0..TASKS {
            let mut storage = self.storage.clone();
            let riverstask = Arc::clone(riversfull);
            let progress = Arc::clone(progress);
            let task = async move {
                for ibatch in 0..BATCHS_TASK {
                    let mut batch = Vec::with_capacity(RIVRS_BATCH);
                    for iriver in 0..RIVRS_BATCH {
                        let i = RIVRS_BATCH * BATCHS_TASK * itask + RIVRS_BATCH * ibatch + iriver;
                        if let Some(observation) = riverstask.get(i) {
                            let equity = observation.equity();
                            let bucket = equity * Abstraction::BUCKETS as f32;
                            let abstraction = Abstraction::from(bucket as u64);
                            batch.push((observation.clone(), abstraction));
                            progress.lock().await.update(observation, equity);
                        }
                    }
                    storage.set_obs_batch(batch).await;
                }
            };
            tasks.push(tokio::task::spawn(task));
        }
        futures::future::join_all(tasks).await;
        self
    }

    ///
    /// things conditional on street: initials, Observation::all
    ///
    /// shared MEMORY state:
    /// Vec<Centroid<(Hash, Histogram)>>
    /// >> established in initials()
    /// >> updated (absorbs histo) after each observation gets mapped to a centroid
    ///
    /// owned MEMORY state:
    /// Vec<Obs>
    /// Map<Obs, Histogram>
    /// >> lookup/move to shared ASYNC if too big in memory
    ///
    /// shared ASYNC state:
    /// Map<Obs , Centroid<Hash>>
    /// >> this could be table
    /// Map<Pair, f32>
    /// >> we could get from db and keep in memory
    ///
    /// each iteration of K-means requires join_all across worker threads
    ///
    /// possibilities can be divided into chunks to handle by thread
    /// think about each thread running as Vec<Obs> -> async (Centroids, Neighbors, Distances)
    pub async fn cluster(&mut self, street: Street) -> &mut Self {
        assert!(street != Street::Rive);
        // maybe predecessors moves to Abstractor
        // this becomes wrapped in a loop over streets
        // for street in Street::iter() { match street { => Obs::preds(s) } }
        let ref possibilities = Observation::all(street);
        let ref mut neighbors = HashMap::<Observation, usize>::with_capacity(possibilities.len());
        let ref mut centroids = self.initials().await;
        self.set_abstractions(centroids, neighbors, possibilities)
            .await;
        self.set_distances(centroids).await;
        self
    }

    async fn set_abstractions(
        &mut self,
        centroids: &mut Vec<Centroid>,
        neighbors: &mut HashMap<Observation, usize>,
        observations: &Vec<Observation>,
    ) {
        const ITERATIONS: usize = 100;
        for _ in 0..ITERATIONS {
            for obs in observations.iter() {
                let histogram = self.storage.get_histogram(obs.clone()).await;
                let ref x = histogram;
                let mut position = 0usize;
                let mut minimium = f32::MAX;
                for (i, centroid) in centroids.iter().enumerate() {
                    let y = centroid.histogram();
                    let emd = self.emd(x, y).await;
                    if emd < minimium {
                        position = i;
                        minimium = emd;
                    }
                }
                // storage.neighbors.async_distances(obs, centroid);
                /* we can also wait til the last iteration to insert this i believe */
                neighbors.insert(obs.clone(), position);
                centroids
                    .get_mut(position)
                    .expect("position in range")
                    .expand(histogram);
            }
        }
        // some optimization about keeping neighbors in database rather than hashamp
        for (observation, index) in neighbors.iter() {
            let centroid = centroids.get(*index).expect("index in range");
            let abs = centroid.signature();
            let obs = observation.clone();
            self.storage.set_obs(obs, abs).await;
        }
    }

    async fn set_distances(&mut self, centroids: &mut Vec<Centroid>) {
        for centroid in centroids.iter_mut() {
            centroid.shrink();
        }
        for (i, a) in centroids.iter().enumerate() {
            for (j, b) in centroids.iter().enumerate() {
                if i > j {
                    let x = a.signature();
                    let y = b.signature();
                    let xor = Pair::from((x, y));
                    let x = a.histogram();
                    let y = b.histogram();
                    let distance = self.emd(x, y).await;
                    self.storage.set_xor(xor, distance).await;
                }
            }
        }
    }

    /// Earth mover's distance using our precomputed distance metric.
    ///
    ///
    async fn emd(&self, this: &Histogram, that: &Histogram) -> f32 {
        let n = this.size();
        let m = that.size();
        let mut cost = 0.0;
        let mut extra = HashMap::new();
        let mut goals = vec![1.0 / n as f32; n];
        let mut empty = vec![false; n];
        for i in 0..m {
            for j in 0..n {
                if empty[j] {
                    continue;
                }
                let this_key = this.domain()[j];
                let that_key = that.domain()[i];
                let spill = extra
                    .get(that_key)
                    .cloned()
                    .or_else(|| Some(that.weight(that_key)))
                    .expect("key is somewhere");
                if spill == 0f32 {
                    continue;
                }
                let xor = Pair::from((*this_key, *that_key));
                let d = self.storage.get_xor(xor).await;
                let bonus = spill - goals[j];
                if (bonus) < 0f32 {
                    extra.insert(*that_key, 0f32);
                    cost += d * bonus as f32;
                    goals[j] -= bonus as f32;
                } else {
                    extra.insert(*that_key, bonus);
                    cost += d * goals[j];
                    goals[j] = 0.0;
                    empty[j] = true;
                }
            }
        }
        cost
    }
}

struct Progress {
    begin: Instant,
    check: Instant,
    complete: usize,
}

impl Progress {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            complete: 0,
            begin: now,
            check: now,
        }
    }

    fn update(&mut self, river: &Observation, equity: f32) {
        use std::io::Write;
        self.complete += 1;
        if self.complete % 1_000 == 0 {
            let now = Instant::now();
            let total_t = now.duration_since(self.begin);
            let check_t = now.duration_since(self.check);
            self.check = now;
            println!("\x1B4F\x1B[2K{:10} Observations", self.complete);
            println!("\x1B[2K Elapsed: {:.0?}", total_t);
            println!("\x1B[2K Last 1k: {:.0?}", check_t);
            println!(
                "\x1B[2K Mean 1k: {:.0?}",
                (total_t / (self.complete / 1_000) as u32)
            );
            println!("\x1B[2K {} -> {:.3}", river, equity);
            std::io::stdout().flush().unwrap();
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        let now = Instant::now();
        self.complete = 0;
        self.begin = now;
        self.check = now;
    }
}
