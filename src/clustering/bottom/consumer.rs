use super::progress::Progress;
use crate::clustering::bottom::Abstraction;
use crate::clustering::bottom::Observation;
use std::pin::Pin;
use tokio::sync::mpsc::Receiver;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::Type;

pub struct Consumer {
    rx: Receiver<(Observation, Abstraction)>,
    writer: BinaryCopyInWriter,
    client: tokio_postgres::Client,
}

impl Consumer {
    pub async fn new(rx: Receiver<(Observation, Abstraction)>) -> Self {
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

    pub async fn run(mut self) {
        let ref mut writer = self.writer;
        let mut writer = unsafe { Pin::new_unchecked(writer) };
        let mut progress = Progress::new();
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
        self.client
            .execute("COMMIT", &[])
            .await
            .expect("commit transaction");
    }
}

const INIT: &'static str = r#"
            BEGIN;
            CREATE UNLOGGED TABLE IF NOT EXISTS centroid (
                observation BIGINT PRIMARY KEY,
                abstraction BIGINT,
                street      CHAR(1)
            );
            CREATE UNLOGGED TABLE IF NOT EXISTS distance (
                xor         BIGINT PRIMARY KEY,
                distance    FLOAT,
                street      CHAR(1)
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
