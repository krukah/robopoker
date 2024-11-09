use super::bucket::Bucket;
use crate::mccfr::player::Player;
use crate::play::game::Game;

#[derive(Debug)]
pub struct Data {
    game: Game,
    info: Bucket,
}

impl From<(Game, Bucket)> for Data {
    fn from((game, info): (Game, Bucket)) -> Self {
        Self { game, info }
    }
}

impl Data {
    pub fn game(&self) -> &Game {
        &self.game
    }
    // pub fn recall(&self) -> &Recall {
    //     &self.past
    // }
    pub fn bucket(&self) -> &Bucket {
        &self.info
    }
    pub fn player(&self) -> Player {
        Player(self.game.player())
    }
}
