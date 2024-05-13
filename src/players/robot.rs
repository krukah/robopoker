pub struct Robot;

impl Player for Robot {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        todo!()
    }
}

impl Robot {
    fn weight(&self, action: Action) -> u32 {
        match action {
            Action::Fold(_) => 1500,
            Action::Check(_) => 1000,
            Action::Call(..) => 4000,
            Action::Raise(..) => 500,
            Action::Shove(..) => 1,
            _ => 0,
        }
    }
}

impl Debug for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player")
    }
}

use crate::gameplay::player::Player;
use crate::gameplay::{action::Action, hand::Hand, seat::Seat};
use std::fmt::Debug;
