use super::bucket::Recall;
use crate::mccfr::player::Player;
use crate::play::game::Game;

#[derive(Debug)]
pub struct Data {
    game: Game,
    past: Recall,
}

impl From<(Game, Recall)> for Data {
    fn from((game, past): (Game, Recall)) -> Self {
        Self { game, past }
    }
}

impl Data {
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn recall(&self) -> &Recall {
        &self.past
    }
    pub fn player(&self) -> Player {
        Player(self.game.player())
    }
}
