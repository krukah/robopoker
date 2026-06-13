use super::*;
use rbp_mccfr::*;
use rbp_transport::Support;

/// Actions in Kuhn poker.
///
/// Deal edges carry exact cards (for chance nodes).
/// Betting edges are check, bet, call, fold.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum KuhnEdge {
    Deal(Card),
    Check,
    Bet,
    Call,
    Fold,
}

impl Support for KuhnEdge {}
impl CfrEdge for KuhnEdge {}

impl KuhnEdge {
    pub fn is_bet(&self) -> bool {
        matches!(self, Self::Check | Self::Bet | Self::Call | Self::Fold)
    }
}

impl std::fmt::Display for KuhnEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deal(c) => write!(f, "D{c}"),
            Self::Check => write!(f, "X"),
            Self::Bet => write!(f, "B"),
            Self::Call => write!(f, "C"),
            Self::Fold => write!(f, "F"),
        }
    }
}
