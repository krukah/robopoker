use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::encoding::Encoder;
use crate::play::game::Game;

use super::bucket::Bucket;
use super::data::Data;
use super::edge::Edge;
use super::node::Node;
use super::path::Path;

#[derive(Default)]
pub struct Sampler(Encoder);

impl Sampler {
    pub fn load() -> Self {
        Self(Encoder::load())
    }
    pub fn root(&self) -> Data {
        let game = Game::root();
        let present = self.recall(&game);
        let history = Path::default();
        let futures = Path::default();
        let infoset = Bucket::from((history, present, futures));
        Data::from((game, infoset))
    }
    pub fn children(&self, node: &Node) -> Vec<(Data, Edge)> {
        let choices = node.unfold();
        let history = node.subgame();
        let futures = Path::from(choices.clone());
        let history = Path::from(history);
        choices
            .into_iter()
            .map(|e| (e, node.action(e)))
            .map(|(e, a)| (e, node.data().game().apply(a)))
            .map(|(e, g)| (e, g, self.recall(&g)))
            .map(|(e, g, h)| (e, g, Bucket::from((history, h, futures))))
            .map(|(e, g, i)| (e, Data::from((g, i))))
            .map(|(e, d)| (d, e))
            .inspect(|(d, e)| log::info!("{} {} {}", e, d.bucket(), d.game()))
            .collect()
    }
    pub fn recall(&self, game: &Game) -> Abstraction {
        self.0
            .abstraction(&Isomorphism::from(Observation::from(game)))
    }
}
