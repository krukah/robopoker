use super::{
    action::{Action, Actor},
    seat::Seat,
};

pub struct Player {
    pub seat: Seat,
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
