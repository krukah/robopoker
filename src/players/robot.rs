pub struct Robot;

impl Player for Robot {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        self.policy(seat, hand).choose()
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

    fn policy(&self, seat: &Seat, hand: &Hand) -> Policy {
        Policy {
            choices: seat
                .valid_actions(hand)
                .iter()
                .map(|a| Choice {
                    action: *a,
                    weight: self.weight(*a),
                })
                .collect(),
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
use crate::strategy::policy::{Choice, Policy};
use std::fmt::Debug;
