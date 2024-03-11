#[derive(Clone, Debug)]
pub enum Player {
    Human(Hole),
    Robot(Hole),
}

impl Player {
    pub fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        let policies = self.policies(seat, hand);
        self.choose(policies)
    }

    fn weight(&self, action: Action) -> u32 {
        match action {
            Action::Fold(_) => 15,
            Action::Check(_) => 10,
            Action::Call(..) => 40,
            Action::Raise(..) => 5,
            Action::Shove(..) => 0,
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

use super::{action::Action, game::Hand, seat::Seat};
use crate::{cards::hole::Hole, solver::policy::Policy};
use rand::{thread_rng, Rng};
