use super::data::Data;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::encoding::Encoder;
use crate::play::game::Game;

#[derive(Default)]
pub struct Sampler(Encoder);

impl Sampler {
    pub fn load() -> Self {
        Self(Encoder::load())
    }
    pub fn root(&self) -> Data {
        let game = Game::root();
        let info = self.recall(&game);
        Data::from((game, info))
    }
    pub fn recall(&self, game: &Game) -> Abstraction {
        self.0
            .abstraction(&Isomorphism::from(Observation::from(game)))
    }
}
