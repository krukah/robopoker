#![allow(unused)]

use super::abstraction::Abstraction;
use super::layer::Layer;
use crate::cards::observation::Observation;
use crate::cfr::bucket::Bucket;
use crate::cfr::data::Data;
use crate::cfr::edge::Edge;
use crate::play::game::Game;
use std::collections::BTreeMap;

pub struct Abstractor(BTreeMap<Observation, Abstraction>);

impl Abstractor {
    pub async fn download() -> Self {
        todo!("try to load ~1.2TB of Obs -> Abs map into memory, lmao")
    }
    pub async fn upload() {
        Layer::outer()
            .await
            .save() // river
            .await
            .inner()
            .save() // turn
            .await
            .inner()
            .save() // flop
            .await
            .inner()
            .save() // preflop
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
        let observation = Observation::from(game);
        let abstraction = self.abstraction(observation);
        Bucket::from(abstraction)
    }
    fn abstraction(&self, ref observation: Observation) -> Abstraction {
        self.0
            .get(observation)
            .copied()
            .expect("download should have all observations")
    }
}
