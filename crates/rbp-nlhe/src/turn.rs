//! NLHE turn type: player indicator.
use rbp_gameplay::Turn;
use rbp_mccfr::*;
use rbp_transport::Support;

/// NLHE turn indicator for CFR traversal.
///
/// Newtype wrapper around gameplay `Turn` for NLHE-specific CFR.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NlheTurn(Turn);

impl NlheTurn {
    /// True if this is a player decision point.
    pub fn is_choice(&self) -> bool {
        self.0.is_choice()
    }
    /// Returns the terminal turn indicator.
    pub fn terminal() -> Self {
        Self(Turn::Terminal)
    }
}

impl Support for NlheTurn {}
impl CfrTurn for NlheTurn {
    fn chance() -> Self {
        Self(Turn::Chance)
    }
    fn terminal() -> Self {
        Self(Turn::Terminal)
    }
}

impl From<Turn> for NlheTurn {
    fn from(turn: Turn) -> Self {
        Self(turn)
    }
}
impl From<NlheTurn> for Turn {
    fn from(turn: NlheTurn) -> Self {
        turn.0
    }
}
impl AsRef<Turn> for NlheTurn {
    fn as_ref(&self) -> &Turn {
        &self.0
    }
}
impl From<usize> for NlheTurn {
    fn from(player: usize) -> Self {
        Self(Turn::from(player))
    }
}

impl std::fmt::Display for NlheTurn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Turn::from(*self))
    }
}
