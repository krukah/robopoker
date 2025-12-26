use super::*;
use crate::mccfr::TrainingStats;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio_postgres::Client;

pub struct Pool {
    workers: Vec<Worker>,
    started: Instant,
    checked: Mutex<Instant>,
}

impl Pool {
    pub async fn new(client: Arc<Client>) -> Self {
        Self {
            workers: (0..num_cpus::get())
                .map(|_| client.clone())
                .map(Worker::new)
                .collect(),
            started: Instant::now(),
            checked: Mutex::new(Instant::now()),
        }
    }
    pub async fn step(&self) {
        futures::future::join_all(self.workers.iter().map(|w| w.step())).await;
    }
    pub fn checkpoint(&self) -> Option<String> {
        let mut last = self.checked.lock().unwrap();
        if last.elapsed() >= crate::TRAINING_LOG_INTERVAL {
            *last = Instant::now();
            Some(self.stats())
        } else {
            None
        }
    }
}

impl TrainingStats for Pool {
    fn epoch(&self) -> usize {
        self.workers.iter().map(|w| w.epoch()).sum()
    }
    fn nodes(&self) -> usize {
        self.workers.iter().map(|w| w.nodes()).sum()
    }
    fn infos(&self) -> usize {
        self.workers.iter().map(|w| w.infos()).sum()
    }
    fn elapsed(&self) -> std::time::Duration {
        self.started.elapsed()
    }
}
