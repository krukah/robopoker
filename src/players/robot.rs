pub struct Robot;
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

    fn policies(&self, seat: &Seat, hand: &Hand) -> Vec<Policy> {
        seat.valid_actions(hand)
            .iter()
            .map(|a| Policy {
                action: a.clone(),
                weight: self.weight(a.clone()),
            })
            .collect()
    }

    fn choose(&self, policies: Vec<Policy>) -> Action {
        let total = policies.iter().map(|p| p.weight).sum();
        let roll = thread_rng().gen_range(0..total);
        let mut sum = 0;
        for policy in policies.iter() {
            sum += policy.weight;
            if roll < sum {
                return policy.action.clone();
            }
        }
        unreachable!()
    }
}

impl Player for Robot {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        let policies = self.policies(seat, hand);
        self.choose(policies)
    }
}

impl Debug for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player")
    }
}

use crate::gameplay::player::Player;
use crate::gameplay::{action::Action, hand::Hand, seat::Seat};
use crate::solver::policy::Policy;
use rand::{thread_rng, Rng};
use std::fmt::Debug;
