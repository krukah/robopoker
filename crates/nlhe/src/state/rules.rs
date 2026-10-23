//! The pure NLHE game.
use super::*;
use kicker::*;
use mccfr::*;

/// Heads-up NLHE with chip-exact, chance-resolved moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NlheRules(Game);

impl Rules for NlheRules {
    type Move = Action;
    type Turn = NlheTurn;

    fn turn(&self) -> NlheTurn {
        NlheTurn::from(self.0.turn())
    }

    fn apply(&self, m: Action) -> Self {
        Self(self.0.apply(m))
    }
}

impl From<Game> for NlheRules {
    fn from(game: Game) -> Self {
        Self(game)
    }
}

impl From<NlheRules> for Game {
    fn from(rules: NlheRules) -> Self {
        rules.0
    }
}

impl AsRef<Game> for NlheRules {
    fn as_ref(&self) -> &Game {
        &self.0
    }
}
