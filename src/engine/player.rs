pub trait Player: Debug {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action;
}
pub struct Robot;
impl Robot {
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

use super::{action::Action, hand::Hand, seat::Seat};
use crate::solver::policy::Policy;
use rand::{thread_rng, Rng};
use std::fmt::Debug;

pub struct Human;
impl Human {}
impl Player for Human {
    fn act(&self, seat: &Seat, hand: &Hand) -> Action {
        let actions = seat.valid_actions(hand);
        println!("CHOOSE VALID ACTION:");
        for (i, action) in actions.iter().enumerate() {
            println!(" - {}: {}", i, action);
        }
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        println!("");
        let index = input.trim().parse::<usize>().unwrap();
        actions
            .get(index)
            .unwrap_or(&Action::Fold(seat.seat_id))
            .clone()
    }
}
impl Debug for Human {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Human")
    }
}
