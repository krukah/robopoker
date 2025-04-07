#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Game(u8);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Edge {
    R,
    P,
    S,
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Turn {
    P1,
    P2,
    Terminal,
}
impl crate::transport::support::Support for Edge {}
impl crate::cfr::traits::edge::Edge for Edge {}
impl crate::cfr::traits::turn::Turn for Turn {}
impl crate::cfr::traits::info::Info for Turn {
    type E = Edge;
    type T = Turn;
    fn choices(&self) -> Vec<Self::E> {
        vec![Edge::R, Edge::P, Edge::S]
    }
}
impl crate::cfr::traits::game::Game for Game {
    type E = Edge;
    type T = Turn;
    fn root() -> Self {
        Self(0)
    }

    fn turn(&self) -> Self::T {
        match self.0 {
            00..=00 => Turn::P1,
            01..=03 => Turn::P2,
            04..=12 => Turn::Terminal,
            _ => unreachable!(),
        }
    }

    fn apply(&self, edge: Self::E) -> Self {
        match (self.0, edge) {
            // P1 moves
            (00, Edge::R) => Self(01),
            (00, Edge::P) => Self(02),
            (00, Edge::S) => Self(03),
            // P2 moves
            (01, Edge::R) => Self(04),
            (01, Edge::P) => Self(05),
            (01, Edge::S) => Self(06),
            (02, Edge::R) => Self(07),
            (02, Edge::P) => Self(08),
            (02, Edge::S) => Self(09),
            (03, Edge::R) => Self(10),
            (03, Edge::P) => Self(11),
            (03, Edge::S) => Self(12),
            // terminal nodes
            _ => unreachable!(),
        }
    }
    fn payoff(&self, turn: Self::T) -> crate::Utility {
        todo!()
    }
}

struct Sampler;
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
        _: &crate::cfr::structs::tree::Tree<Self::T, Self::E, Self::G, Self::I>,
        (_, game, _): crate::cfr::types::branch::Branch<Self::E, Self::G>,
    ) -> Self::I {
        use crate::cfr::traits::game::Game;
        game.turn()
    }
}
