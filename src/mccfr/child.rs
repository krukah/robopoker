use super::edge::Edge;
use crate::cards::observation::Observation;
use crate::mccfr::odds::Odds;
use crate::play::action::Action;
use crate::play::game::Game;

/// Represents a child node in the game tree, containing both
/// the game state and the action that led to it
#[derive(Debug, Clone)]
pub struct Leaf {
    from: Action,
    game: Game,
}

impl Leaf {
    pub fn into(self) -> (Game, Action) {
        (self.game, self.from)
    }
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn action(&self) -> &Action {
        &self.from
    }
    pub fn observation(&self) -> Observation {
        Observation::from(self.game())
    }
    pub fn edge(&self) -> Edge {
        if let &Action::Raise(bet) = self.action() {
            Edge::from(Odds::from((bet, self.game().pot() - bet)))
        } else {
            Edge::from(self.action().clone())
        }
    }
}

impl From<(Game, Action)> for Leaf {
    fn from((game, action): (Game, Action)) -> Self {
        Self { game, from: action }
    }
}
