use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::observation::Observation;
use crate::cards::street::Street;
use std::collections::HashMap;
use std::vec;

/// Abstract representation of street used to generate hierarchical clusters.
///
/// Each street is generated from the next lowest level of abstraction,
/// with the river being generated from scratch.
pub struct Layer {
    street: Street,
    metric: HashMap<Pair, f32>,
    clusters: HashMap<Observation, Abstraction>,
}

impl Layer {
    /// The River layer is at the bottom of the hierarchy, and is generated from scratch.
    pub fn river() -> Self {
        Self {
            clusters: River::clusters(),
            metric: River::distance(),
            street: Street::Rive,
        }
    }

    /// Generate a layer from the next lower-level of abstraction.
    pub fn upper(lower: &Self) -> Self {
        let histograms = lower.histograms();
        let ref centroids = lower.centroids(histograms.values().collect());
        Self {
            street: lower.street.prev(),
            metric: lower.metric(centroids),
            clusters: lower.clusters(centroids, histograms),
        }
    }

    /// Generate a histogram for each Observation in the next layer.
    ///
    /// We associate each upper-layer Observation with a lower-layer histogram
    /// of its children. Which then allows for us to define a distance metric (Earth mover's
    /// distance) on the non-ordinal set of Histograms. Which then allows us to
    /// cluster the next layer using the lower-layer's centroids.
    fn histograms(&self) -> HashMap<Observation, Histogram> {
        Observation::predecessors(self.street)
            .into_iter()
            .map(|ref pred| (*pred, self.histogram(pred)))
            .collect::<HashMap<_, _>>()
    }

    /// Lookup abstractions of this Observation's children and create a histogram.
    ///
    /// The children of an Observation are the lower-layer's Observation. These
    /// can be mapped to the lower-layer's abstractions via the `clusters` HashMap.
    /// We map reduce into a Histogram, which is the upper layer's Observation decomposed
    /// into its lower-layer's abstractions.
    fn histogram(&self, predecessor: &Observation) -> Histogram {
        Histogram::from(
            predecessor
                .successors()
                .map(|ref succ| self.abstraction(succ))
                .collect::<Vec<_>>(),
        )
    }

    /// Lookup precomputed Abstraction of an Observation in the lower-layer.
    fn abstraction(&self, observation: &Observation) -> Abstraction {
        self.clusters
            .get(observation)
            .copied()
            .expect("we should have computed signatures previously")
    }

    /// Lookup precomputed distance between two Abstractions in the lower-layer.
    fn distance(&self, a: &Abstraction, b: &Abstraction) -> f32 {
        let ref index = Pair::from((*a, *b));
        self.metric
            .get(index)
            .copied()
            .expect("we should have computed distances previously")
    }

    /// Precompute the distance between each pair of centroids in the lower-layer.
    fn metric(&self, centroids: &Vec<Histogram>) -> HashMap<Pair, f32> {
        println!("Calculating {} distances...", self.street);
        let mut distances = HashMap::new();
        for (i, a) in centroids.iter().enumerate() {
            for (j, b) in centroids.iter().enumerate() {
                if i > j {
                    let key = Pair::from((Abstraction::from(a), Abstraction::from(b)));
                    let distance = self.emd(a, b);
                    distances.insert(key, distance);
                }
            }
        }
        distances
    }

    /// Cluster the next layer using the lower-layer's centroids + netric.
    #[rustfmt::skip]
    fn clusters(&self, centroids: &Vec<Histogram>, histograms: HashMap<Observation, Histogram>) -> HashMap<Observation, Abstraction> {
        println!("Clustering {}...", self.street);
        let mut abstractions = HashMap::new();
        for (observation, ref histogram) in histograms {
            let mut minimium = f32::MAX;
            let mut neighbor = histogram;
            for ref centroid in centroids {
                let distance = self.emd(histogram, centroid);
                if distance < minimium {
                    minimium = distance;
                    neighbor = centroid;
                }
            }
            abstractions.insert(observation, Abstraction::from(neighbor));
        }
        abstractions
    }

    /// Earth mover's distance using our precomputed distance metric.
    ///
    /// We use the heuristic method of "spilling" goals across buckets until
    /// there are no more goals to spill.
    /// Potential-Aware Imperfect-Recall Abstraction with Earth Mover’s Distance in
    /// Imperfect-Information Games;
    /// Ganzfried et. al 2014
    fn emd(&self, this: &Histogram, that: &Histogram) -> f32 {
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
                let d = self.distance(this_key, that_key);
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

    /// Cluster via k-meansusing our custom distance metric.
    ///
    /// K is determined by the number of centroids in our initial guess. We should
    /// implement k-means++ in the future. Iterations are fixed at comptime.
    fn centroids(&self, histograms: Vec<&Histogram>) -> Vec<Histogram> {
        const ITERATIONS: usize = 100;
        let mut centroids = self.guesses();
        let k = centroids.len();
        for _ in 0..ITERATIONS {
            let mut clusters: Vec<Vec<&Histogram>> = vec![vec![]; k];
            for x in histograms.iter() {
                let mut position = 0usize;
                let mut minimium = f32::MAX;
                for (i, y) in centroids.iter().enumerate() {
                    let distance = self.emd(x, y);
                    if distance < minimium {
                        minimium = distance;
                        position = i;
                    }
                }
                clusters
                    .get_mut(position)
                    .expect("position in range")
                    .push(x);
            }
            centroids = clusters
                .into_iter()
                .map(|points| Histogram::centroid(points))
                .collect::<Vec<Histogram>>();
        }
        centroids
    }

    /// Initial guesses for this layer
    fn guesses(&self) -> Vec<Histogram> {
        todo!("implement k-means++ initialization")
    }
}

/// River layer is generated from scratch, so we give it it's own type.
struct River;
impl River {
    /// Cluster the river layer using showdown equity.
    ///
    /// Showdown equity is the probability of winning the hand if the
    /// opponents cards are turned face up. These are the only Abstractions
    /// derived as    f32 -> u8  -> Abstraction, compared to the distribution-
    /// derived Histogram -> u64 -> Abstraction
    fn clusters() -> HashMap<Observation, Abstraction> {
        println!("Clustering {}...", Street::Rive);
        Observation::predecessors(Street::Show)
            .into_iter()
            .map(|obs| (obs, Abstraction::from(obs)))
            .collect::<HashMap<_, _>>()
    }

    /// Distances between river Equities are calculated as the absolute difference in equity.
    ///
    /// These are precomputed without any clustering because we can just have a lookup table
    /// of all (BUCKETS choose 2) pairwise distances. Precomputing them is more conveienient,
    /// albeit less efficient, than calculating them on the fly, because it allows us to recursively
    /// use Layer::distance to calculate the distance between any two Abstractions at any given Layer.
    fn distance() -> HashMap<Pair, f32> {
        println!("Calculating {} distances...", Street::Rive);
        let mut metric = HashMap::new();
        let equities = Abstraction::buckets();
        for (i, a) in equities.iter().enumerate() {
            for (j, b) in equities.iter().enumerate() {
                if i > j {
                    let key = Pair::from((*a, *b));
                    let distance = (i - j) as f32;
                    metric.insert(key, distance);
                }
            }
        }
        metric
    }
}

/// A unique identifier for a pair of abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct Pair(u64);
impl From<(Abstraction, Abstraction)> for Pair {
    fn from((a, b): (Abstraction, Abstraction)) -> Self {
        Self(u64::from(a) ^ u64::from(b))
    }
}
impl From<Pair> for i64 {
    fn from(pair: Pair) -> Self {
        pair.0 as i64
    }
}

impl Layer {
    /// Async persistence to storage.
    ///
    pub async fn save(&self, pool: &sqlx::PgPool) {
        println!("Saving {}...", self.street);
        // begin tx
        let mut tx = pool
            .begin()
            .await
            .expect("crossing fingers, begin transaction");
        // insert metric
        for (pair, distance) in self.metric.iter() {
            sqlx::query(
                r#"
                INSERT INTO metric  (xor, distance, street)
                VALUES              ($1, $2, $3)"#,
            )
            .bind(i64::from(*pair))
            .bind(f32::from(*distance))
            .bind(self.street as i64)
            .execute(&mut tx)
            .await
            .expect("insert metric");
        }
        // insert clusters
        for (observation, abstraction) in self.clusters.iter() {
            sqlx::query(
                r#"
                INSERT INTO cluster (observation, abstraction, street)
                VALUES              ($1, $2, $3)"#,
            )
            .bind(i64::from(*observation))
            .bind(i64::from(*abstraction))
            .bind(self.street as i64)
            .execute(&mut tx)
            .await
            .expect("insert cluster");
        }
        // commit tx
        tx.commit()
            .await
            .expect("crossing fingers, commit transaction");
    }
}
