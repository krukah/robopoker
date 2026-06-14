//! NLHE game type: poker game state.
use super::*;
use cowboys::*;
use kicker::*;
use pokerkit::Utility;
use regret::*;

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
    /// Observation from a specific player's perspective.
    pub fn sweat_at(&self, position: usize) -> Observation {
        self.0.sweat_at(position)
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
    /// Applies an edge to the game state.
    ///
    /// Handles canonical replay divergence: when replaying edge history
    /// from a different context (e.g., AIVAT inference), snapped chip
    /// amounts can cause the canonical game to reach Chance or Terminal
    /// at a different point than the actual game.
    ///
    /// - Choice edge at Chance: auto-deal through chance nodes first
    /// - Draw edge at Choice: skip (canonical hasn't reached that street)
    /// - Any edge at Terminal: return terminal (canonical ended early)
    fn apply(&self, edge: Self::E) -> Self {
        let edge = Edge::from(edge);
        let mut game = self.0;
        if game.turn() == Turn::Terminal {
            return Self(game);
        }
        if edge.is_choice() {
            while game.turn() == Turn::Chance {
                game = game.force_apply(game.reveal());
            }
            if game.turn() == Turn::Terminal {
                return Self(game);
            }
        }
        if edge.is_chance() && game.turn() != Turn::Chance {
            return Self(game);
        }
        let action = game.actionize(edge);
        let action = game.snap(action);
        Self(game.apply(action))
    }

    fn depth(&self) -> usize {
        self.0.street() as usize
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
