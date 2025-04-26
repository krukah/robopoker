use super::edge::Edge;
use super::turn::Turn;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game(u8);

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
            (00, Edge::R) => Self(01),
            (00, Edge::P) => Self(02),
            (00, Edge::S) => Self(03),
            (01, Edge::R) => Self(04),
            (01, Edge::P) => Self(05),
            (01, Edge::S) => Self(06),
            (02, Edge::R) => Self(07),
            (02, Edge::P) => Self(08),
            (02, Edge::S) => Self(09),
            (03, Edge::R) => Self(10),
            (03, Edge::P) => Self(11),
            (03, Edge::S) => Self(12),
            _ => unreachable!(),
        }
    }

    fn payoff(&self, turn: Self::T) -> crate::Utility {
        const P_WIN: crate::Utility = 1.;
        const S_WIN: crate::Utility = 2.; // we can modify payoffs to verify convergence
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
