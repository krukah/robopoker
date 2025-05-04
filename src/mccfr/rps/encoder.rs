use super::edge::Edge;
use super::game::Game;
use super::solver::RPS;
use super::turn::Turn;
use crate::mccfr::structs::tree::Tree;
use crate::mccfr::types::branch::Branch;

impl crate::mccfr::traits::encoder::Encoder for RPS {
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
        use crate::mccfr::traits::game::Game;
        game.turn()
    }
}
