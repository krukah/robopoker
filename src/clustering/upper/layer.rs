use super::histogram::Histogram;
use super::metric::Metric;
use super::projection::Projection;
use super::xor::Pair;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::bottom::consumer::Consumer;
use crate::clustering::bottom::producer::Producer;
use crate::clustering::bottom::progress::Progress;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::Type;
use tokio_postgres::Client;

pub struct Layer {
    street: Street,
    metric: HashMap<Pair, f32>,
    observations: HashMap<Observation, (Histogram, Abstraction)>,
    abstractions: HashMap<Abstraction, (Histogram, Histogram)>,
}

impl Layer {
    /// async download from database to create initial River layer.
    pub async fn bottom() -> Self {
        let layer = Self {
            street: Street::Rive,
            metric: Self::bottom_metric(),
            observations: Self::bottom_observations().await,
            abstractions: HashMap::default(),
        };
        layer.upload().await;
        layer
    }

    /// Yield the next layer of abstraction by kmeans clustering
    /// TODO; make this async and persist to database after each layer
    pub async fn raise(self) -> Self {
        let mut layer = Self {
            street: self.street.prev(),
            metric: self.raise_metric(),
            observations: self.raise_observations(),
            abstractions: self.raise_abstractions(),
        };
        layer.kmeans(100);
        layer.upload().await;
        layer
    }

    /// Run kmeans iterations.
    /// Presumably, we have been generated by a previous layer, with the exception of Bottom == River.
    /// After the base case, we trust that our observations, abstractions, and metric are correctly populated.
    fn kmeans(&mut self, iterations: usize) {
        println!("clustering {} {}", self.street, self.observations.len());
        for _ in 0..iterations {
            for (_, (data, last)) in self.observations.iter_mut() {
                let mut nearests = f32::MAX;
                let mut neighbor = Abstraction::default();
                for (abstraction, (mean, _)) in self.abstractions.iter_mut() {
                    let distance = self.metric.emd(data, mean);
                    if distance < nearests {
                        nearests = distance;
                        neighbor = abstraction.clone();
                    }
                }
                self.abstractions
                    .get_mut(&neighbor)
                    .expect("key from iteration, not default")
                    .0
                    .absorb(data);
                let _ = std::mem::replace(last, neighbor);
            }
        }
    }

    /// Calculate and return the metric using EMD distances between abstractions
    fn raise_metric(&self) -> HashMap<Pair, f32> {
        let ref centroids = self.abstractions;
        let mut metric = HashMap::new();
        for (i, (x, _)) in centroids.iter().enumerate() {
            for (j, (y, _)) in centroids.iter().enumerate() {
                if i > j {
                    let index = Pair::from((x, y));
                    let ref x = centroids.get(x).expect("kmeans histogram").0;
                    let ref y = centroids.get(y).expect("kmeans histogram").0;
                    let distance = self.metric.emd(x, y);
                    metric.insert(index, distance);
                }
            }
        }
        metric
    }

    /// Generate all possible obersvations. Assign them to arbitrary abstractions. They will be overwritten during kmeans iterations. We start from River which comes from database from equity abstractions.
    #[rustfmt::skip]
    fn raise_observations(&self) -> HashMap<Observation, (Histogram, Abstraction)> {
        Observation::all(self.street.prev())
            .into_iter()
            .map(|upper| (upper, (self.observations.project(upper), Abstraction::default())))
            .collect()
    }

    /// K Means++ implementation yields initial histograms. Abstractions are random and require uniqueness.
    #[rustfmt::skip]
    fn raise_abstractions(&self) -> HashMap<Abstraction, (Histogram, Histogram)> {
        println!("initializing abstraction centroids");
        // 0. Initialize data structures
        let mut initials = Vec::new();
        let ref mut histograms = self.observations.values().map(|(histogram, _)| histogram);
        let ref mut rng = rand::thread_rng();
        use rand::distributions::Distribution;
        use rand::distributions::WeightedIndex;
        use rand::seq::SliceRandom;
        // 1. Choose 1st centroid randomly from the dataset
        let sample = histograms
            .collect::<Vec<&Histogram>>()
            .choose(rng)
            .expect("non-empty lower observations")
            .to_owned()
            .clone();
        initials.push(sample);
        // 2. Choose nth centroid with probability proportional to squared distance of nearest neighbors
        const K: usize = 100;
        while initials.len() < K {
            let distances = histograms
                .map(|histogram| initials
                    .iter()
                    .map(|initial| self.metric.emd(initial, histogram))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .expect("find minimum")
                )
                .map(|min| min * min)
                .collect::<Vec<f32>>();
            let choice = WeightedIndex::new(distances)
                .expect("valid weights")
                .sample(rng);
            let sample = histograms
                .nth(choice)
                .expect("shared index with lowers")
                .clone();
            initials.push(sample);
        }
        // 3. Collect histograms and label with arbitrary (random) Abstractions
        initials
            .into_iter()
            .map(|mean| (Abstraction::random(), (mean, Histogram::default())))
            .collect::<HashMap<_, _>>()
    }

    /// Generate the  baseline metric between equity bucket abstractions. Keeping the u64->f32 conversion is fine for distance since it preserves distance
    fn bottom_metric() -> HashMap<Pair, f32> {
        let mut metric = HashMap::new();
        for i in 0..Abstraction::EQUITIES as u64 {
            for j in i..Abstraction::EQUITIES as u64 {
                let distance = (j - i) as f32;
                let ref i = Abstraction::from(i);
                let ref j = Abstraction::from(j);
                let index = Pair::from((i, j));
                metric.insert(index, distance);
            }
        }
        metric
    }

    // construct observation -> abstraction map via equity calculations
    async fn bottom_observations() -> HashMap<Observation, (Histogram, Abstraction)> {
        println!("clustering bottom layer");
        let ref observations = Arc::new(Observation::all(Street::Rive));
        let (tx, rx) = tokio::sync::mpsc::channel::<(Observation, Abstraction)>(1024);
        let consumer = Consumer::new(rx);
        let consumer = tokio::spawn(consumer.run());
        let producers = (0..num_cpus::get())
            .map(|i| Producer::new(i, tx.clone(), observations.clone()))
            .map(|p| tokio::spawn(p.run()))
            .collect::<Vec<_>>();
        std::mem::drop(tx);
        futures::future::join_all(producers).await;
        consumer.await.expect("equity mapping task completes")
    }

    /// Upload to database
    async fn upload(&self) {
        let ref url = std::env::var("DATABASE_URL").expect("DATABASE_URL in environment");
        let (ref client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls)
            .await
            .expect("connect to database");
        tokio::spawn(connection);
        // maybe wrap this in a transaction?
        self.upload_distance(client).await;
        self.upload_centroid(client).await;
    }

    /// Truncate the database tables
    #[allow(unused)]
    async fn truncate(client: &Client) {
        client
            .batch_execute(
                r#"
                    DROP TABLE IF EXISTS centroid;
                    DROP TABLE IF EXISTS distance;
                    CREATE UNLOGGED TABLE centroid (
                        observation BIGINT PRIMARY KEY,
                        abstraction BIGINT
                    );
                    CREATE UNLOGGED TABLE distance (
                        xor         BIGINT PRIMARY KEY,
                        distance    REAL
                    );
                    TRUNCATE TABLE centroid;
                    TRUNCATE TABLE distance;
                "#,
            )
            .await
            .expect("begin transaction");
    }

    /// Upload centroid data to the database
    /// would love to be able to FREEZE table for initial river COPY
    async fn upload_centroid(&self, client: &Client) {
        let sink = client
            .copy_in(
                r#" 
                    COPY centroid (
                        observation,
                        abstraction
                    )
                    FROM STDIN BINARY;
                "#,
            )
            .await
            .expect("get sink for COPY transaction");
        let ref mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::INT8]);
        let mut writer = unsafe { Pin::new_unchecked(writer) };
        let mut progress = Progress::new(self.observations.len());
        for (observation, (_, abstraction)) in self.observations.iter() {
            let ref observation = i64::from(observation.clone()); // zero copy if impl ToSql
            let ref abstraction = i64::from(abstraction.clone()); // zero copy if impl ToSql
            writer
                .as_mut()
                .write(&[observation, abstraction])
                .await
                .expect("write row into heap");
            progress.tick();
        }
        writer
            .finish()
            .await
            .expect("complete centroid COPY transaction");
    }

    /// Upload distance data to the database
    /// would love to be able to FREEZE table for initial river COPY
    async fn upload_distance(&self, client: &Client) {
        let sink = client
            .copy_in(
                r#"
                    COPY distance (
                        xor,
                        distance
                    )
                    FROM STDIN BINARY;
                "#,
            )
            .await
            .expect("get sink for COPY transaction");
        let ref mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::FLOAT4]);
        let mut writer = unsafe { Pin::new_unchecked(writer) };
        let mut progress = Progress::new(self.metric.len());
        for (pair, distance) in self.metric.iter() {
            let ref pair = i64::from(pair.clone()); // zero copy if impl ToSql
            writer
                .as_mut()
                .write(&[pair, distance])
                .await
                .expect("write row into heap");
            progress.tick();
        }
        writer
            .finish()
            .await
            .expect("complete distance COPY transaction");
    }
}
