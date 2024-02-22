use crate::cards::hole::Hole;

use super::{
    action::{Action, Actor},
    game::Game,
    seat::Seat,
};

pub struct Player {
    pub hand: Hole,
    pub game: &'static Game,
    pub seat: &'static Seat,
}

impl Player {
    pub fn new() -> Player {
        todo!()
    }
}

impl Actor for Player {
    fn act(&self) -> Action {
        todo!()
    }
}
