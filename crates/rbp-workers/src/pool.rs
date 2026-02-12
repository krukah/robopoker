use super::*;
use rbp_core::*;
use rbp_mccfr::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio_postgres::Client;

/// Pool of distributed training workers.
///
/// Uses Pluribus configuration via [`Worker`].
pub struct Pool {
    workers: Vec<Worker>,
    started: Instant,
    prior: Mutex<(Instant, usize)>,
}

impl Pool {
    pub async fn new(client: Arc<Client>) -> Self {
        let now = Instant::now();
        Self {
            workers: (0..num_cpus::get())
                .map(|_| client.clone())
                .map(Worker::new)
                .collect(),
            started: now,
            prior: Mutex::new((now, 0)),
        }
    }
    pub async fn step(&self) {
        futures::future::join_all(self.workers.iter().map(|w| w.step())).await;
    }
    pub fn checkpoint(&self) -> Option<String> {
        let mut prior = self.prior.lock().unwrap();
        if prior.0.elapsed() >= TRAINING_LOG_INTERVAL {
            let secs = prior.0.elapsed().as_secs().max(1) as f64;
            let curr = self.infos();
            let rate = (curr - prior.1) as f64 / secs;
            *prior = (Instant::now(), curr);
            Some(format!(
                "{:<20}{:<20}{:<20}{:<20}",
                format!("epoch {}", self.epoch()),
                format!("nodes {}", self.nodes()),
                format!("infos {}", curr),
                format!("I/sec {:.1}", rate),
            ))
        } else {
            None
        }
    }
}

impl Progress for Pool {
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
