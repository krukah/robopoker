use super::storage::Storage;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MemoryLookup {
    cluster: Arc<Mutex<HashMap<Observation, Abstraction>>>,
    metrics: Arc<Mutex<HashMap<Pair, f32>>>,
}

impl Storage for MemoryLookup {
    async fn new() -> Self {
        Self {
            cluster: Arc::new(Mutex::new(HashMap::with_capacity(2_800_000_000))),
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    async fn set_obs(&mut self, obs: Observation, abs: Abstraction) {
        self.cluster.lock().await.insert(obs, abs);
    }
    async fn set_xor(&mut self, xor: Pair, distance: f32) {
        self.metrics.lock().await.insert(xor, distance);
    }
    async fn get_obs(&self, ref obs: Observation) -> Abstraction {
        self.cluster
            .lock()
            .await
            .get(obs)
            .copied()
            .expect("obs to have been populated")
    }
    async fn get_xor(&self, ref xor: Pair) -> f32 {
        self.metrics
            .lock()
            .await
            .get(xor)
            .copied()
            .expect("xor to have been populated")
    }
}
