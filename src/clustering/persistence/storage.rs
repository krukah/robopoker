use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;

#[allow(async_fn_in_trait)]
pub trait Storage: Clone {
    async fn new() -> Self;
    async fn set_obs(&mut self, obs: Observation, abs: Abstraction);
    async fn set_xor(&mut self, xor: Pair, distance: f32);
    async fn get_obs(&self, obs: Observation) -> Abstraction;
    async fn get_xor(&self, xor: Pair) -> f32;

    /// ~1Kb download
    /// this could possibly be implemented as a join?
    /// fml a big Vec<> of these is gonna have to fit
    /// in memory for the centroid calculation
    async fn get_histogram(&self, obs: Observation) -> Histogram {
        let mut abstractions = Vec::new();
        let successors = obs.outnodes();
        for succ in successors {
            let abstraction = self.get_obs(succ).await;
            abstractions.push(abstraction);
        }
        Histogram::from(abstractions)
    }
}
