use crate::mccfr::bucket::Bucket;
use crate::mccfr::edge::Edge;
use crate::mccfr::player::Player;
use crate::play::continuation::Continuation;
use crate::play::game::Game;

/// pot
/// n_bets
/// observation
/// abstraction
/// rotation
pub struct Data {
    game: Game,
    bucket: Bucket,
}

impl From<(Game, Bucket)> for Data {
    fn from((game, bucket): (Game, Bucket)) -> Self {
        Self { game, bucket }
    }
}

impl Data {
    pub fn root() -> Self {
        Self {
            game: Game::root(),
            bucket: todo!(),
        }
    }
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }
    pub fn player(&self) -> Player {
        match self.game.chooser() {
            a @ Continuation::Decision(_) => Player::Choice(a),
            _ => Player::Chance,
        }
    }
    pub fn edges(&self) -> Vec<Edge> {
        self.game
            .options()
            .into_iter()
            .map(|a| Edge::from(a))
            .collect()
    }
}
