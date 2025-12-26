use crate::gameplay::*;
use crate::gameroom::*;
use rand::seq::IndexedRandom;

/// Example CPU player that chooses randomly from legal actions.
/// Demonstrates synchronous decision-making in async context.
pub struct Fish;

#[async_trait::async_trait]
impl Player for Fish {
    async fn decide(&mut self, recall: &Recall) -> Action {
        let ref mut rng = rand::rng();
        recall
            .head()
            .legal()
            .choose(rng)
            .copied()
            .expect("non empty legal actions conditional on being asked to move")
    }

    async fn notify(&mut self, _: &Event) {}
}
