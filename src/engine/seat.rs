#[derive(Debug, Clone)]
pub struct Seat {
    pub sunk: u32,
    pub stack: u32,
    pub status: BetStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BetStatus {
    Playing,
    Shoved,
    Folded,
}

impl Seat {
    pub fn new(stack: u32) -> Seat {
        Seat {
            stack,
            sunk: 0,
            status: BetStatus::Playing,
        }
    }
}

impl Actor for Seat {
    fn act(&self) -> Action {
        todo!()
    }
}

use super::action::{Action, Actor};
