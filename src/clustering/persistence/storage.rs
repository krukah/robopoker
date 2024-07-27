use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;

#[allow(async_fn_in_trait)]
pub trait Storage {
    async fn new() -> Self;
    async fn set_obs(&mut self, obs: Observation, abs: Abstraction);
    async fn set_xor(&mut self, xor: Pair, distance: f32);
    async fn get_obs(&self, obs: Observation) -> Abstraction;
    async fn get_xor(&self, xor: Pair) -> f32;
    async fn get_histogram(&self, obs: Observation) -> Histogram;
}
