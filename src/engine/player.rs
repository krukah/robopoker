use super::{
    action::{Action, Actor},
    seat::Seat,
};
use std::rc::Rc;

pub struct Player {
    pub index: usize,
    pub seat: Rc<Seat>,
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
