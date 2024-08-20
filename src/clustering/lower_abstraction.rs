use crate::cards::street::Street;
use crate::clustering::equivalence::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;
use std::sync::Arc;
///
///
///
///
///
///
///
///
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

const TASKS: usize = 8;
const RIVERS: usize = 2_809_475_760;
const RIVERS_PER_TASK: usize = RIVERS / TASKS;

struct EquitySource {
    tx: Sender<(Observation, Abstraction)>,
    shard: usize,
    observations: Arc<Vec<Observation>>,
}
impl EquitySource {
    fn new(
        shard: usize,
        tx: Sender<(Observation, Abstraction)>,
        observations: Arc<Vec<Observation>>,
    ) -> Self {
        Self {
            tx,
            shard,
            observations,
        }
    }

    async fn run(self) {
        let beg = self.shard * RIVERS_PER_TASK;
        let end = self.shard * RIVERS_PER_TASK + RIVERS_PER_TASK;
        for index in beg..end {
            if let Some(observation) = self.observations.get(index) {
                let abstraction = Abstraction::from(observation);
                let observation = observation.clone();
                self.tx
                    .send((observation, abstraction))
                    .await
                    .expect("channel to be open");
            } else {
                return;
            }
        }
    }
}

///
///
///
///
///
///
///
///
///

const BATCH_MIN: usize = 10_000;
const BATCH_MAX: usize = 10_000 * 2;

struct EquitySink {
    rx: Receiver<(Observation, Abstraction)>,
    buffer: Vec<(Observation, Abstraction)>,
    client: tokio_postgres::Client,
    progress: Progress,
}
impl EquitySink {
    async fn new(rx: Receiver<(Observation, Abstraction)>) -> Self {
        const QUERY: &str = r#"
            CREATE UNLOGGED TABLE IF NOT EXISTS centroid (
                observation BIGINT PRIMARY KEY,
                abstraction BIGINT,
                street CHAR(1)
            );
            CREATE UNLOGGED TABLE IF NOT EXISTS distance (
                xor BIGINT PRIMARY KEY,
                distance FLOAT,
                street CHAR(1)
            );
            TRUNCATE TABLE centroid;
            TRUNCATE TABLE distance;
        "#;
        let buffer = Vec::with_capacity(BATCH_MAX);
        let progress = Progress::new();
        let ref url = std::env::var("DATABASE_URL").expect("DATABASE_URL in environment");
        let (client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls)
            .await
            .expect("to connect to database");
        tokio::spawn(connection);
        client
            .batch_execute(QUERY)
            .await
            .expect("to intialize tables");
        Self {
            rx,
            buffer,
            client,
            progress,
        }
    }

    async fn run(mut self) {
        while let Some((obs, abs)) = self.rx.recv().await {
            self.progress.increment();
            self.buffer.push((obs, abs));
            if self.buffer.len() == BATCH_MIN {
                self.flush().await;
            }
        }
        if self.buffer.len() > 0 {
            println!("Flushing remaining buffer");
            self.flush().await;
        }
    }

    async fn flush(&mut self) {
        let sink = self
            .client
            .copy_in(
                r#"
                    COPY centroid
                    (street, observation, abstraction)
                    FROM STDIN BINARY
                "#,
            )
            .await
            .expect("to begin COPY transaction");
        let writer = tokio_postgres::binary_copy::BinaryCopyInWriter::new(
            sink,
            &[
                tokio_postgres::types::Type::CHAR, // street
                tokio_postgres::types::Type::INT8, // observation
                tokio_postgres::types::Type::INT8, // abstraction
            ],
        );
        futures::pin_mut!(writer);
        for (obs, abs) in self.buffer.drain(..) {
            let ref street = obs.street() as i8;
            let ref observation = i64::from(obs);
            let ref abstraction = i64::from(abs);
            writer
                .as_mut()
                .write(&[street, observation, abstraction])
                .await
                .expect("to write row");
        }
        writer.finish().await.expect("to complete COPY transaction");
    }
}

///
///
///
///
///
///
///
///
///
///
///

const CHANNEL_SIZE: usize = TASKS * 2;

pub struct LowerAbstractionAlgo;
impl LowerAbstractionAlgo {
    pub async fn river() {
        let mut tasks = Vec::with_capacity(TASKS);
        let ref observations = Arc::new(Observation::all(Street::Rive));
        let (tx, rx) = tokio::sync::mpsc::channel::<(Observation, Abstraction)>(CHANNEL_SIZE);
        let uploader = EquitySink::new(rx).await;
        tasks.push(tokio::spawn(uploader.run()));
        for task in 0..TASKS {
            let calculator = EquitySource::new(task, tx.clone(), observations.clone());
            tasks.push(tokio::task::spawn(calculator.run()));
        }
        futures::future::join_all(tasks).await;
    }
}

///
///
///
///
///
use std::collections::HashMap;

const K: usize = 100;

#[allow(unused)]
pub struct UpperAbstractionAlgo {
    metric: HashMap<Pair, f64>,
    prev_centroids: [Histogram; K],
    next_centroids: [Histogram; K],
}

#[allow(unused)]
impl UpperAbstractionAlgo {
    fn distance(&self, a: &Abstraction, b: &Abstraction) -> f64 {
        let xor = Pair::from((a.clone(), b.clone()));
        *self
            .metric
            .get(&xor)
            .expect("distance to be pre-calculated")
    }

    fn swap(&mut self) {
        let ref mut prev = self.prev_centroids;
        let ref mut next = self.next_centroids;
        std::mem::swap(prev, next);
        for centroid in next.iter_mut() {
            centroid.weights.clear();
        }
    }
}
///
///
///
///
///
///
///
use std::time::Instant;
pub struct Progress {
    begin: Instant,
    check: Instant,
    complete: u32,
}
impl Progress {
    const CHECKPOINT: u32 = 10_000;
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            begin: now,
            check: now,
            complete: 0,
        }
    }
    pub fn increment(&mut self) {
        use std::io::Write;
        self.complete += 1;
        if self.complete % Self::CHECKPOINT == 0 {
            let now = Instant::now();
            let total_t = now.duration_since(self.begin);
            let check_t = now.duration_since(self.check);
            self.check = now;
            print!("\x1B[4A"); // Move cursor up 4 lines (for 4 lines of output)
            print!("\x1B[0J"); // Clear from cursor to end of screen
            println!("Elapsed: {:.0?}", total_t);
            #[rustfmt::skip]
            println!("Mean Freq:{:>10.0}", self.complete as f32 / total_t.as_secs_f32());
            #[rustfmt::skip]
            println!("Last Freq:{:>10.0}", BATCH_MIN as f32 / check_t.as_secs_f32());
            #[rustfmt::skip]
            println!("{:10}{:>10.1}%", self.complete, (self.complete as f32 / RIVERS as f32) * 100.0);
            std::io::stdout().flush().unwrap();
        }
    }
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.complete = 0;
        self.begin = now;
        self.check = now;
    }
}
