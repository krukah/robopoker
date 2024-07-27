use super::abstraction::Abstraction;
use super::histogram::Centroid;
use super::histogram::Histogram;
use super::observation::Observation;
use super::persistence::postgres::PostgresLookup;
use super::persistence::storage::Storage;
use super::xor::Pair;
use crate::cards::street::Street;
use std::collections::HashMap;
use std::time::Instant;
use std::vec;

pub struct Abstractor {
    storage: PostgresLookup,
}

impl Abstractor {
    pub async fn new() -> Self {
        Self {
            storage: PostgresLookup::new().await,
        }
    }

    pub async fn learn() {
        todo!("the whole thing across all layers streets")
    }

    async fn guesses(&self) -> Vec<Centroid> {
        todo!("implement k-means++ initialization")
    }

    /// Save the river
    ///
    #[rustfmt::skip]
    pub async fn river(&mut self) {
        let begin = Instant::now();
        let mut check = begin;

        println!("Clustering {}...", Street::Rive);
        for (i, river) in Observation::all(Street::Rive).into_iter().enumerate() {
            let equity = river.equity();
            let quantile = equity * Abstraction::BUCKETS as f32;
            let abstraction = Abstraction::from(quantile as u64);
            self.storage.set_obs(river, abstraction).await;

            if i % 1_000_000 == 0 {
                let now = Instant::now();
                let check_duration = now.duration_since(check);
                let total_duration = now.duration_since(begin);
                println!("{} observations clustered", i);
                println!("Time for last million: {:.2?}", check_duration);
                println!("Total time elapsed: {:.2?}", total_duration);
                println!("Average time per million: {:.2?}", total_duration / i as u32 / 1_000_000 as u32);
                check = now;
            }
        }
    }

    pub async fn cluster(mut self, street: Street) -> Self {
        // maybe predecessors moves to Abstractor
        // this becomes wrapped in a loop over streets
        // for street in Street::iter() { match street { => Obs::preds(s) } }
        let ref possibilities = Observation::all(street);
        let ref mut neighbors = HashMap::<Observation, usize>::with_capacity(possibilities.len());
        let ref mut centroids = self.guesses().await;
        self.kmeans(centroids, neighbors, possibilities).await;
        self.upsert(centroids, neighbors).await;
        self.insert(centroids).await;
        self
    }

    #[rustfmt::skip]
    async fn kmeans(&self, centroids: &mut Vec<Centroid>, neighbors: &mut HashMap<Observation, usize>, observations: &Vec<Observation>) {
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
                neighbors.insert(obs.clone(), position);
                centroids
                    .get_mut(position)
                    .expect("position in range")
                    .expand(histogram);
            }
        }
    }

    async fn upsert(&mut self, centroids: &[Centroid], neighbors: &HashMap<Observation, usize>) {
        for (observation, index) in neighbors.iter() {
            let centroid = centroids.get(*index).expect("index in range");
            let abs = centroid.signature();
            let obs = observation.clone();
            self.storage.set_obs(obs, abs).await;
        }
    }

    async fn insert(&mut self, centroids: &mut Vec<Centroid>) {
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
