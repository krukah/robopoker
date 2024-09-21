use crate::cfr::bucket::Bucket;
use crate::cfr::edge::Edge;
use crate::cfr::player::Player;
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
            Continuation::Decision(position) => Player::Choice(Continuation::Decision(position)),
            Continuation::Awaiting(_) => Player::Chance,
            Continuation::Terminal => Player::Chance,
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
