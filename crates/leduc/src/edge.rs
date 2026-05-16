use super::*;
use rbp_mccfr::*;
use rbp_transport::Support;

/// Actions in Leduc Hold'em.
///
/// Deal edges carry exact cards (for chance nodes).
/// Betting edges are the standard poker actions.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LeducEdge {
    Deal(Card),
    Fold,
    Check,
    Call,
    Raise,
}

impl Support for LeducEdge {}
impl CfrEdge for LeducEdge {}

impl LeducEdge {
    pub fn is_bet(&self) -> bool {
        matches!(self, Self::Fold | Self::Check | Self::Call | Self::Raise)
    }
}

impl std::fmt::Display for LeducEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deal(c) => write!(f, "D{}", c),
            Self::Fold => write!(f, "F"),
            Self::Check => write!(f, "X"),
            Self::Call => write!(f, "C"),
            Self::Raise => write!(f, "R"),
        }
    }
}
