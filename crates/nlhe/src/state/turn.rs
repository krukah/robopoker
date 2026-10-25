//! NLHE turn type: player indicator.
use kicker::Turn;
use kicker::Witness;
use mccfr::*;
use monge::Support;

/// NLHE turn indicator for CFR traversal: a newtype over `kicker::Turn`.
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
    /// The recall's own seat — subject of the *signalled* range.
    pub fn narrator(recall: &Witness) -> Self {
        Self::from(recall.turn())
    }
    /// The seat opposite the recall's — subject of the *opponent* range.
    pub fn opponent(recall: &Witness) -> Self {
        Self::narrator(recall).other()
    }
    /// The other seat in a two-player game. Panics on chance/terminal turns.
    fn other(self) -> Self {
        match self.0 {
            Turn::Choice(0) => Self::from(1_usize),
            Turn::Choice(1) => Self::from(0_usize),
            _ => unreachable!("two-player game requires Choice turn"),
        }
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

    fn players() -> usize {
        pokerkit::N
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
