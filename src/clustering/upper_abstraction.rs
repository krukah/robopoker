use super::histogram::Centroid;
use super::histogram::Histogram;
use super::observation::Observation;
use super::xor::Pair;
use crate::cards::street::Street;
use std::collections::HashMap;
use std::vec;

type Lookup = super::postgres::PostgresLookup;

pub struct UpperAbstractionAlgo(Lookup);

impl UpperAbstractionAlgo {
    pub async fn new() -> Self {
        Self(Lookup::new().await)
    }

    async fn initials(&self) -> Vec<Centroid> {
        todo!("implement k-means++ initialization")
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
                let ref histogram = self.0.get_histogram(obs.clone()).await;
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
                    .merge(histogram);
            }
        }
        // some optimization about keeping neighbors in database rather than hashamp
        for (observation, index) in neighbors.iter() {
            let centroid = centroids.get(*index).expect("index in range");
            let abs = centroid.signature();
            let obs = observation.clone();
            self.0.set_centroid(obs, abs).await;
        }
    }

    async fn set_distances(&mut self, centroids: &mut Vec<Centroid>) {
        for (i, a) in centroids.iter().enumerate() {
            for (j, b) in centroids.iter().enumerate() {
                if i > j {
                    let x = a.signature();
                    let y = b.signature();
                    let xor = Pair::from((x, y));
                    let x = a.histogram();
                    let y = b.histogram();
                    let distance = self.emd(x, y).await;
                    self.0.set_distance(xor, distance).await;
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
                // weird clone/copy semantics
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
                let xor = Pair::from((this_key.clone(), that_key.clone()));
                let d = self.0.get_distance(xor).await;
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

// struct KMeans {
//     t: usize,
//     data: HashMap<Observation, Histogram>,
//     centroids: [Histogram; K],
// }
// impl KMeans {
//     async fn new(street: Street) -> Self {
//         todo!("grab all histograms from database. use join or parallelized select")
//     }

//     async fn cluster(&self) {}

//     async fn initials(&self) -> [Centroid; K] {
//         todo!("k-means initialization")
//     }
// }

// use std::sync::Arc;
// const K: usize = 10;
// type Index = usize;

// async fn neighbor(x: &Histogram, centroids: Arc<[Histogram; K]>) -> Index {
//     let mut position = 0usize;
//     let mut minimium = f32::MAX;
//     for (i, y) in centroids.iter().enumerate() {
//         let emd = self.emd(x, y).await;
//         if emd < minimium {
//             position = i;
//             minimium = emd;
//         }
//     }
//     position
// }

// Arc<HashMap<Observation, Histogram>>
//  Arc<
