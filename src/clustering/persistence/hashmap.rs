use super::storage::Storage;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;
use std::collections::HashMap;

pub struct HashMapLookup {
    cluster: HashMap<Observation, Abstraction>,
    metrics: HashMap<Pair, f32>,
}

impl Storage for HashMapLookup {
    async fn new() -> Self {
        Self {
            cluster: HashMap::new(),
            metrics: HashMap::new(),
        }
    }
    async fn set_obs(&mut self, obs: Observation, abs: Abstraction) {
        self.cluster.insert(obs, abs);
    }
    async fn set_xor(&mut self, xor: Pair, distance: f32) {
        self.metrics.insert(xor, distance);
    }
    async fn get_obs(&self, ref obs: Observation) -> Abstraction {
        self.cluster
            .get(obs)
            .copied()
            .expect("obs to have been populated")
    }
    async fn get_xor(&self, ref xor: Pair) -> f32 {
        self.metrics
            .get(xor)
            .copied()
            .expect("xor to have been populated")
    }
    async fn get_histogram(&self, obs: Observation) -> Histogram {
        let mut abstractions = Vec::new();
        let successors = obs.successors();
        for succ in successors {
            let abstraction = self.get_obs(succ).await;
            abstractions.push(abstraction);
        }
        Histogram::from(abstractions)
    }
}
