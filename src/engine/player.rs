pub struct Player {
    pub position: usize,
    pub hole: Hole,
}

impl Player {
    pub fn new(seat: &Seat) -> Player {
        Player {
            hole: Hole::new(),
            position: seat.position,
        }
    }
}

impl Actor for Player {
    fn act(&self, _game: &Game) -> Action {
        Action::Fold
    }
}
use super::{
    action::{Action, Actor},
    game::Game,
    seat::Seat,
};
use crate::cards::hole::Hole;
