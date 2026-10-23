use super::*;

/// One step of a [`Recall::walk`]: the state a history passed through, whose turn it was there,
/// and the canonical edge its action maps to.
///
/// The `game` is the real state *before* the action, carrying the community cards actually
/// dealt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stride {
    turn: Turn,
    game: Game,
    edge: Edge,
}

impl Stride {
    /// Whose turn it was at this state — [`Turn::Chance`] at a card deal.
    pub fn turn(&self) -> Turn {
        self.turn
    }
    /// The state before the action, holding the cards actually dealt.
    pub fn game(&self) -> Game {
        self.game
    }
    /// The canonical ([`Game::edgify`]) edge for the action taken here.
    pub fn edge(&self) -> Edge {
        self.edge
    }
}

impl From<(Turn, Game, Edge)> for Stride {
    fn from((turn, game, edge): (Turn, Game, Edge)) -> Self {
        Self { turn, game, edge }
    }
}
