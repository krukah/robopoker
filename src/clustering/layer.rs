use super::abstraction::Abstraction;
use super::histogram::{Centroid, Histogram};
use super::observation::Observation;
use super::xor::Pair;
use crate::cards::street::Street;
use sqlx::query;
use std::collections::HashMap;
use std::vec;

pub struct Layer {
    street: Street,
    db: sqlx::PgPool,
    // predecessors
    // neighbors
    // centroids
}

impl Layer {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self {
            street: Street::Rive,
            db,
        }
    }

    async fn guesses(&self) -> Vec<Centroid> {
        todo!("implement k-means++ initialization")
    }

    /// Save the river
    ///
    pub async fn river(&self) {
        println!("Clustering {}...", Street::Rive);
        for obs in Observation::predecessors(Street::Show) {
            let abs = Abstraction::from(obs);
            self.set_obs(obs, abs).await
        }
        println!("Calculating {} distances...", Street::Rive);
        let equities = Abstraction::buckets();
        for (i, a) in equities.iter().enumerate() {
            for (j, b) in equities.iter().enumerate() {
                if i > j {
                    let xor = Pair::from((a.clone(), b.clone()));
                    let distance = (i - j) as f32;
                    self.set_xor(xor, distance).await;
                }
            }
        }
    }

    pub async fn cluster(mut self) -> Self {
        let ref observations = Observation::predecessors(self.street);
        let ref mut neighbors = HashMap::<Observation, usize>::with_capacity(observations.len());
        let ref mut centroids = self.guesses().await;
        self.kmeans(centroids, neighbors, observations).await;
        self.upsert(centroids, neighbors).await;
        self.insert(centroids).await;
        self.street = self.street.prev();
        self
    }

    #[rustfmt::skip]
    async fn kmeans(&self, centroids: &mut Vec<Centroid>, neighbors: &mut HashMap<Observation, usize>, observations: &Vec<Observation>) {
        const ITERATIONS: usize = 100;
        for _ in 0..ITERATIONS {
            for obs in observations.iter() {
                let histogram = self.decompose(obs.clone()).await;
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

    async fn upsert(&self, centroids: &[Centroid], neighbors: &HashMap<Observation, usize>) {
        for (observation, index) in neighbors.iter() {
            let centroid = centroids.get(*index).expect("index in range");
            let abs = centroid.signature();
            let obs = observation.clone();
            self.set_obs(obs, abs).await;
        }
    }

    async fn insert(&self, centroids: &mut Vec<Centroid>) {
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
                    self.set_xor(xor, distance).await;
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
                let d = self.get_xor(xor).await;
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

    // The following methods encapsulate database lookups and inserts.

    /// ~1Kb download
    /// this could possibly be implemented as a join?
    /// fml a big Vec<> of these is gonna have to fit
    /// in memory for the centroid calculation
    async fn decompose(&self, pred: Observation) -> Histogram {
        let mut abstractions = Vec::new();
        let successors = pred.successors();
        for succ in successors {
            let abstraction = self.get_obs(succ).await;
            abstractions.push(abstraction);
        }
        Histogram::from(abstractions)
    }

    /// Insert row into cluster table
    async fn set_obs(&self, obs: Observation, abs: Abstraction) {
        sqlx::query(
            r#"
                INSERT INTO cluster (observation, abstraction, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (observation)
                DO UPDATE SET       abstraction = $2"#,
        )
        .bind(i64::from(obs))
        .bind(i64::from(abs))
        .bind(self.street as i64)
        .execute(&self.db)
        .await
        .expect("database insert: cluster");
    }

    /// Insert row into metric table
    async fn set_xor(&self, xor: Pair, distance: f32) {
        sqlx::query(
            r#"
                INSERT INTO metric  (xor, distance, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (xor)
                DO UPDATE SET       distance = $2"#,
        )
        .bind(i64::from(xor))
        .bind(f32::from(distance))
        .bind(self.street as i64)
        .execute(&self.db)
        .await
        .expect("database insert: metric");
    }

    /// Query Observation -> Abstraction table
    async fn get_obs(&self, obs: Observation) -> Abstraction {
        let abs = query!(
            r#"
                SELECT abstraction
                FROM cluster
                WHERE observation = $1 AND street = $2"#,
            i64::from(obs),
            self.street as i64
        )
        .fetch_one(&self.db)
        .await
        .expect("to respond to cluster query")
        .abstraction
        .expect("to have computed cluster previously");
        Abstraction::from(abs)
    }

    /// Query Pair -> f32 table
    async fn get_xor(&self, xor: Pair) -> f32 {
        let distance = query!(
            r#"
                SELECT distance
                FROM metric
                WHERE xor = $1 AND street = $2"#,
            i64::from(xor),
            self.street as i64
        )
        .fetch_one(&self.db)
        .await
        .expect("to respond to metric query")
        .distance
        .expect("to have computed metric previously");
        distance as f32
    }
}
