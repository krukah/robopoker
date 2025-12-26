use super::*;
use crate::mccfr::*;
use crate::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpsGame(u8);

impl TreeGame for RpsGame {
    type E = RpsEdge;
    type T = RpsTurn;

    fn root() -> Self {
        Self(0)
    }

    fn turn(&self) -> Self::T {
        match self.0 {
            00..=00 => RpsTurn::P1,
            01..=03 => RpsTurn::P2,
            04..=12 => RpsTurn::Terminal,
            _ => unreachable!(),
        }
    }

    fn apply(&self, edge: Self::E) -> Self {
        match (self.0, edge) {
            (00, RpsEdge::R) => Self(01),
            (00, RpsEdge::P) => Self(02),
            (00, RpsEdge::S) => Self(03),
            (01, RpsEdge::R) => Self(04),
            (01, RpsEdge::P) => Self(05),
            (01, RpsEdge::S) => Self(06),
            (02, RpsEdge::R) => Self(07),
            (02, RpsEdge::P) => Self(08),
            (02, RpsEdge::S) => Self(09),
            (03, RpsEdge::R) => Self(10),
            (03, RpsEdge::P) => Self(11),
            (03, RpsEdge::S) => Self(12),
            _ => unreachable!(),
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        const P_WIN: Utility = 1.;
        const S_WIN: Utility = P_WIN * ASYMMETRIC_UTILITY;
        let direction = match turn {
            RpsTurn::P1 => 0. + 1.,
            RpsTurn::P2 => 0. - 1.,
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
