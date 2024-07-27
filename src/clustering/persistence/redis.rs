use super::storage::Storage;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;
use redis::AsyncCommands;

pub struct RedisLookup {
    client: redis::Client,
}

impl Storage for RedisLookup {
    async fn new() -> Self {
        const REDIS_URL: &str = "redis://localhost:6379";
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| String::from(REDIS_URL));
        let client = redis::Client::open(url).expect("Redis client to connect");
        Self { client }
    }
    async fn set_obs(&mut self, obs: Observation, abs: Abstraction) {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("cluster:{}", i64::from(obs));
        conn.set::<String, i64, redis::Value>(key, i64::from(abs))
            .await
            .expect("Redis set: cluster");
    }
    async fn set_xor(&mut self, xor: Pair, distance: f32) {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("metric:{}", i64::from(xor));
        conn.set::<String, f32, redis::Value>(key, distance)
            .await
            .expect("Redis set: metric");
    }
    async fn get_obs(&self, obs: Observation) -> Abstraction {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("cluster:{}", i64::from(obs));
        let abs: i64 = conn.get(key).await.expect("Redis get: cluster");
        Abstraction::from(abs)
    }
    async fn get_xor(&self, xor: Pair) -> f32 {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("metric:{}", i64::from(xor));
        let distance: String = conn.get(key).await.expect("Redis get: metric");
        distance.parse().expect("Valid f32")
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
