use super::{edge::Edge, game::Game, turn::Turn};
use crate::cfr::structs::tree::Tree;
use crate::cfr::types::branch::Branch;

pub struct Sampler;

impl crate::cfr::traits::sampler::Sampler for Sampler {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;

    fn seed(&self, _: &Self::G) -> Self::I {
        Turn::P1
    }

    fn info(
        &self,
        _: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (_, game, _): Branch<Self::E, Self::G>,
    ) -> Self::I {
        use crate::cfr::traits::game::Game;
        game.turn()
    }
}
