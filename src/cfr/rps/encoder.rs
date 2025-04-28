use super::edge::Edge;
use super::game::Game;
use super::turn::Turn;
use crate::cfr::structs::tree::Tree;
use crate::cfr::types::branch::Branch;

pub struct Encoder;

impl crate::cfr::traits::encoder::Encoder for Encoder {
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
        (_, ref game, _): Branch<Self::E, Self::G>,
    ) -> Self::I {
        crate::cfr::traits::game::Game::turn(game)
    }
}
