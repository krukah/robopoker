use crate::cards::observation::Observation;
use crate::clustering::encoding::Odds;
use crate::play::action::Action;
use crate::play::game::Game;
use crate::{Probability, Utility};

use super::edge::Edge;

/// Represents a child node in the game tree, containing both
/// the game state and the action that led to it
#[derive(Debug, Clone)]
pub struct Child {
    game: Game,
    action: Action,
}

impl Child {
    pub fn into(self) -> (Game, Action) {
        (self.game, self.action)
    }
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn observation(&self) -> Observation {
        Observation::from(self.game())
    }
    pub fn odds(&self) -> Odds {
        if let Action::Raise(bet) = self.action() {
            let share = *bet as Utility / (self.game().pot() - bet) as Utility;
            let index = Odds::GRID
                .map(|o| Probability::from(o)) // pre-sorted
                .binary_search_by(|p| p.partial_cmp(&share).expect("not NaN"))
                .unwrap_or_else(|i| i.saturating_sub(1)); // Fallback to the closest lower index
            Odds::GRID[index]
        } else {
            unreachable!("only raise actions can be converted to Odds")
        }
    }
    pub fn edge(&self) -> Edge {
        if let Action::Raise(_) = self.action() {
            Edge::from(self.odds())
        } else {
            Edge::from(self.action().clone())
        }
    }
}

impl From<(Game, Action)> for Child {
    fn from((game, action): (Game, Action)) -> Self {
        Self { game, action }
    }
}
