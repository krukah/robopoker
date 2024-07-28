use super::storage::Storage;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;
use redis::AsyncCommands;

#[derive(Clone)]
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
        let key = i64::from(obs);
        self.client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection")
            .set::<i64, i64, redis::Value>(key, i64::from(abs))
            .await
            .expect("Redis set: cluster");
    }
    async fn set_xor(&mut self, xor: Pair, distance: f32) {
        let key = i64::from(xor);
        self.client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection")
            .set::<i64, f32, redis::Value>(key, distance)
            .await
            .expect("Redis set: metric");
    }
    async fn get_obs(&self, obs: Observation) -> Abstraction {
        let key = i64::from(obs);
        let abs: i64 = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection")
            .get(key)
            .await
            .expect("Redis get: cluster");
        Abstraction::from(abs)
    }
    async fn get_xor(&self, xor: Pair) -> f32 {
        let key = i64::from(xor);
        self.client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection")
            .get::<i64, f32>(key)
            .await
            .expect("Redis get: metric")
    }
    async fn set_obs_batch(&mut self, _: Vec<(Observation, Abstraction)>) {
        todo!("redis batch insert")
    }
    async fn set_xor_batch(&mut self, _: Vec<(Pair, f32)>) {
        todo!("redis batch insert")
    }
}
