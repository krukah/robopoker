use super::*;
use rbp_mccfr::*;
use rbp_transport::Support;

/// Player or terminal indicator for RPS.
///
/// The game proceeds: P1 chooses → P2 chooses → Terminal (payoff computed).
/// In RPS, the turn serves as turn, public state, private state, and info set
/// since each player has exactly one decision point with no hidden information.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RpsTurn {
    /// Player 1's turn (root of game tree).
    P1,
    /// Player 2's turn (after P1 has chosen).
    P2,
    /// Terminal state (both players have chosen).
    Terminal,
}

impl From<usize> for RpsTurn {
    fn from(player: usize) -> Self {
        match player {
            0 => Self::P1,
            1 => Self::P2,
            _ => panic!("RPS only has 2 players"),
        }
    }
}

impl Support for RpsTurn {}
impl CfrTurn for RpsTurn {
    fn chance() -> Self {
        Self::Terminal
    }

    fn terminal() -> Self {
        Self::Terminal
    }

    fn players() -> usize {
        2
    }
}

impl CfrPublic for RpsTurn {
    type E = RpsEdge;
    type T = RpsTurn;

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<> {
        match self {
            Self::Terminal => vec![],
            _ => vec![RpsEdge::R, RpsEdge::P, RpsEdge::S],
        }
        .into_iter()
    }

    fn subgame(&self) -> Vec<Self::E> {
        vec![]
    }
}

impl CfrInfo for RpsTurn {
    type Y = ();
    type X = RpsTurn;
    type E = RpsEdge;
    type T = RpsTurn;

    fn public(&self) -> Self::X {
        *self
    }

    fn secret(&self) -> Self::Y {}
}

impl std::fmt::Display for RpsTurn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
