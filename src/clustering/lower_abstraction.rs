use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::Type;

const TASKS: usize = 8;
const RIVERS: usize = 2_809_475_760;
const RIVERS_PER_TASK: usize = RIVERS / TASKS;

struct Producer {
    tx: Sender<(Observation, Abstraction)>,
    shard: usize,
    rivers: Arc<Vec<Observation>>,
}

impl Producer {
    fn new(
        shard: usize,
        tx: Sender<(Observation, Abstraction)>,
        rivers: Arc<Vec<Observation>>,
    ) -> Self {
        Self { tx, shard, rivers }
    }

    async fn run(self) {
        let beg = self.shard * RIVERS_PER_TASK;
        let end = self.shard * RIVERS_PER_TASK + RIVERS_PER_TASK;
        for index in beg..end {
            if let Some(observation) = self.rivers.get(index) {
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

struct Consumer {
    rx: Receiver<(Observation, Abstraction)>,
    writer: BinaryCopyInWriter,
    client: tokio_postgres::Client,
}

impl Consumer {
    async fn new(rx: Receiver<(Observation, Abstraction)>) -> Self {
        const INIT: &'static str = r#"
            BEGIN;
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
        const COPY: &'static str = r#"
            COPY centroid (
                street,
                observation,
                abstraction
            )
            FROM STDIN BINARY FREEZE;
        "#;
        const ROWS: &'static [Type] = &[
            Type::CHAR, // street
            Type::INT8, // observation
            Type::INT8, // abstraction
        ];
        let ref url = std::env::var("DATABASE_URL").expect("DATABASE_URL in environment");
        let (client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls)
            .await
            .expect("connect to database");
        tokio::spawn(connection);
        client
            .batch_execute(INIT)
            .await
            .expect("create and truncate tables");
        let sink = client
            .copy_in(COPY)
            .await
            .expect("get sink for COPY transaction");
        let writer = BinaryCopyInWriter::new(sink, ROWS);
        Self { rx, client, writer }
    }

    async fn run(mut self) {
        let client = self.client;
        let mut progress = Progress::new();
        let ref mut writer = self.writer;
        let mut writer = unsafe { Pin::new_unchecked(writer) };
        while let Some((obs, abs)) = self.rx.recv().await {
            progress.tick();
            let ref street = obs.street() as i8;
            let ref observation = i64::from(obs);
            let ref abstraction = i64::from(abs);
            writer
                .as_mut()
                .write(&[street, observation, abstraction])
                .await
                .expect("write row into heap");
        }
        writer
            .finish()
            .await
            .expect("complete 2.8B rows of COPY transaction");
        client
            .execute("COMMIT", &[])
            .await
            .expect("commit transaction");
    }
}

pub struct RiverAbstraction;
impl RiverAbstraction {
    pub async fn cluster() {
        let mut tasks = Vec::with_capacity(TASKS);
        let ref observations = Arc::new(Observation::all(Street::Rive));
        let (tx, rx) = tokio::sync::mpsc::channel::<(Observation, Abstraction)>(1024);
        let consumer = Consumer::new(rx).await;
        tasks.push(tokio::spawn(consumer.run()));
        for task in 0..TASKS {
            let tx = tx.clone();
            let observations = observations.clone();
            let producer = Producer::new(task, tx, observations);
            tasks.push(tokio::task::spawn(producer.run()));
        }
        futures::future::join_all(tasks).await;
    }
}
/// A struct to track and display progress of a long-running operation.
pub struct Progress {
    begin: Instant,
    delta: Instant,
    complete: u32,
}
impl Progress {
    const CHECKPOINT: usize = 50_000;
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            begin: now,
            delta: now,
            complete: 0,
        }
    }
    pub fn tick(&mut self) {
        self.complete += 1;
        if self.complete % Self::CHECKPOINT as u32 == 0 {
            use std::io::Write;
            let now = Instant::now();
            let total_t = now.duration_since(self.begin);
            let delta_t = now.duration_since(self.delta);
            self.delta = now;
            print!("\x1B[4A"); // Move cursor up 4 lines (for 4 lines of output)
            print!("\x1B[0J"); // Clear from cursor to end of screen
            println!("Elapsed: {:.0?}", total_t);
            #[rustfmt::skip]
            println!("Mean Freq:{:>10.0}", self.complete as f32 / total_t.as_secs_f32());
            #[rustfmt::skip]
            println!("Last Freq:{:>10.0}", Self::CHECKPOINT as f32 / delta_t.as_secs_f32());
            #[rustfmt::skip]
            println!("{:10}{:>10.1}%", self.complete, (self.complete as f32 / RIVERS as f32) * 100.0);
            std::io::stdout().flush().unwrap();
        }
    }
}
