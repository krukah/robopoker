use super::*;
use crate::mccfr::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RpsTurn {
    P1,
    P2,
    Terminal,
}

impl TreeTurn for RpsTurn {
    fn chance() -> Self {
        Self::Terminal
    }
}

impl TreeInfo for RpsTurn {
    type E = RpsEdge;
    type T = RpsTurn;

    fn choices(&self) -> Vec<Self::E> {
        if *self == RpsTurn::Terminal {
            vec![]
        } else {
            vec![RpsEdge::R, RpsEdge::P, RpsEdge::S]
        }
    }
}

impl std::fmt::Display for RpsTurn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
