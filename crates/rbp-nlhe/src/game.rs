//! NLHE game type: poker game state.
use super::*;
use rbp_cards::*;
use rbp_core::Utility;
use rbp_gameplay::*;
use rbp_mccfr::*;

/// NLHE game state for CFR traversal.
///
/// Newtype wrapper around gameplay `Game` for NLHE-specific CFR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NlheGame(Game);

impl NlheGame {
    /// Current betting round (street).
    pub fn street(&self) -> Street {
        self.0.street()
    }
    /// Current observation (hole cards + board).
    pub fn sweat(&self) -> Observation {
        self.0.sweat()
    }
}

impl CfrGame for NlheGame {
    type E = NlheEdge;
    type T = NlheTurn;
    fn root() -> Self {
        Self(Game::root())
    }
    fn turn(&self) -> Self::T {
        NlheTurn::from(self.0.turn())
    }
    fn apply(&self, edge: Self::E) -> Self {
        let action = self.0.actionize(Edge::from(edge));
        let action = self.0.snap(action);
        Self(self.0.apply(action))
    }
    fn payoff(&self, turn: Self::T) -> Utility {
        self.0
            .settlements()
            .get(Turn::from(turn).position())
            .map(|settlement| settlement.won() as Utility)
            .expect("player index in bounds")
    }
}

impl From<Game> for NlheGame {
    fn from(game: Game) -> Self {
        Self(game)
    }
}
impl From<NlheGame> for Game {
    fn from(game: NlheGame) -> Self {
        game.0
    }
}
impl AsRef<Game> for NlheGame {
    fn as_ref(&self) -> &Game {
        &self.0
    }
}
