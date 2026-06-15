use crate::*;
use kicker::*;
use rand::seq::IndexedRandom;

/// Example CPU player that chooses randomly from legal actions.
/// Demonstrates synchronous decision-making in async context.
pub struct Fish;

#[async_trait::async_trait]
impl Player for Fish {
    fn shows(&self) -> bool {
        true
    }

    async fn decide(&mut self, recall: &Witness) -> Action {
        let ref mut rng = rand::rng();
        recall
            .head()
            .legal()
            .into_iter()
            .filter(|a| !a.is_shove())
            .collect::<Vec<_>>()
            .choose(rng)
            .copied()
            .expect("non empty legal actions conditional on being asked to move")
    }
}
