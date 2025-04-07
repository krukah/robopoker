#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game(u8);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edge {
    R,
    P,
    S,
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Turn {
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
        if self == &Turn::Terminal {
            vec![]
        } else {
            vec![Edge::R, Edge::P, Edge::S]
        }
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
        const P_WIN: crate::Utility = R_WIN;
        const R_WIN: crate::Utility = 1.;
        const S_WIN: crate::Utility = 5.; // we can modify payoffs to verify convergence
        let direction = match turn {
            Turn::P1 => 0. + 1.,
            Turn::P2 => 0. - 1.,
            _ => unreachable!(),
        };
        let payoff = match self.0 {
            07 => 0. + P_WIN, // P > R
            05 => 0. - P_WIN, // R < P
            06 => 0. + S_WIN, // R > S
            11 => 0. + S_WIN, // S > P
            10 => 0. - S_WIN, // S < R
            09 => 0. - S_WIN, // P < S
            04 | 08 | 12 => 0.0,
            00..=03 => unreachable!("eval at terminal node, depth > 1"),
            _ => unreachable!(),
        };
        direction * payoff
    }
}

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
        _: &crate::cfr::structs::tree::Tree<Self::T, Self::E, Self::G, Self::I>,
        (_, game, _): crate::cfr::types::branch::Branch<Self::E, Self::G>,
    ) -> Self::I {
        use crate::cfr::traits::game::Game;
        game.turn()
    }
}

use std::collections::BTreeMap;
#[derive(Default)]
pub struct Blueprint {
    epochs: usize,
    encounters: BTreeMap<Turn, BTreeMap<Edge, (crate::Probability, crate::Utility)>>,
}
impl Blueprint {
    pub fn at(&mut self, info: &Turn, edge: &Edge) -> &mut (crate::Probability, crate::Utility) {
        self.encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
    }
    pub fn train() {
        use crate::cfr::traits::trainer::Trainer;
        let mut blueprint = Self::default();
        blueprint.solve();
        log::info!("blueprint: {}", blueprint);
        todo!()
    }
}
impl crate::cfr::traits::profile::Profile for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;
    fn walker(&self) -> Self::T {
        match self.epochs % 2 {
            0 => Turn::P1,
            _ => Turn::P2,
        }
    }
    fn epochs(&self) -> usize {
        self.epochs
    }

    fn weight(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.encounters
            .get(info)
            .and_then(|encounters| encounters.get(edge))
            .map(|(w, _)| *w)
            .unwrap_or(0.)
    }

    fn regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility {
        self.encounters
            .get(info)
            .and_then(|encounters| encounters.get(edge))
            .map(|(_, r)| *r)
            .unwrap_or(0.)
    }

    fn sample(
        &self,
        _: &crate::cfr::structs::node::Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<crate::cfr::types::branch::Branch<Self::E, Self::G>>,
    ) -> Vec<crate::cfr::types::branch::Branch<Self::E, Self::G>> {
        branches
    }
}
impl crate::cfr::traits::trainer::Trainer for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;
    type P = Blueprint;
    type S = Sampler;

    fn encoder(&self) -> &Self::S {
        &Sampler
    }

    fn profile(&self) -> &Self::P {
        self
    }

    fn discount(&self, _: Option<crate::Utility>) -> f32 {
        1.
    }

    fn regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).1
    }

    fn weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).0
    }
}
impl std::fmt::Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Blueprint(epochs: {})", self.epochs)?;
        for (turn, edges) in &self.encounters {
            writeln!(f, "  {:?}:", turn)?;
            for (edge, (prob, regret)) in edges {
                writeln!(
                    f,
                    "    {:?} -> (Prob: {:.2}, Regret: {:.2})",
                    edge, prob, regret
                )?;
            }
        }
        Ok(())
    }
}
