use crate::mccfr::bucket::Bucket;
use crate::mccfr::edge::Edge;
use crate::mccfr::player::Player;
use crate::play::continuation::Transition;
use crate::play::game::Game;

#[derive(Debug)]
pub struct Spot {
    game: Game,
    bucket: Bucket,
}

impl From<(Game, Bucket)> for Spot {
    fn from((game, bucket): (Game, Bucket)) -> Self {
        Self { game, bucket }
    }
}

impl Spot {
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }
    pub fn player(&self) -> Player {
        match self.game.chooser() {
            x @ Transition::Choice(_) => Player::Choice(x),
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