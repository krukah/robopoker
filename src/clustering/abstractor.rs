use super::abstraction::CardAbstraction;
use super::layer::Layer;
use crate::cards::observation::CardObservation;
use crate::cfr::bucket::Bucket;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::play::game::Game;
use std::collections::BTreeMap;

pub struct Abstractor(BTreeMap<CardObservation, CardAbstraction>);

impl Abstractor {
    pub async fn download() -> Self {
        todo!("try to load ~1.2TB of Obs -> Abs map into memory, lmao")
    }
    pub async fn upload() {
        Layer::outer()
            .await
            .upload() // river
            .await
            .inner()
            .upload() // turn
            .await
            .inner()
            .upload() // flop
            .await
            .inner()
            .upload() // preflop
            .await;
    }
    pub fn children(&self, game: &Game) -> Vec<(Data, Edge)> {
        game.options()
            .into_iter()
            .map(|action| (game.imagine(action), action))
            .map(|(g, a)| (self.data(g), Edge::from(a)))
            .collect()
    }

    fn data(&self, game: Game) -> Data {
        let bucket = self.bucket(&game);
        Data::from((game, bucket))
    }
    fn bucket(&self, game: &Game) -> Bucket {
        let observation = CardObservation::from(game);
        let abstraction = self.card_abstraction(observation);
        Bucket::from(abstraction)
    }
    fn card_abstraction(&self, ref observation: CardObservation) -> CardAbstraction {
        self.0
            .get(observation)
            .copied()
            .expect("download should have all observations")
    }
    fn path_abstraction(&self, _: Vec<&Edge>) -> PathAbstraction {
        todo!("pseudoharmonic action mapping for path abstraction")
    }
}

struct PathAbstraction;
