#[derive(Debug, Clone)]
pub struct Seat {
    pub position: usize,
    pub hole: Hole,
    pub actor: Rc<dyn Player>, // Weak ?
    pub stack: u32,
    pub stake: u32,
    pub status: BetStatus,
}
impl Seat {
    pub fn new(stack: u32, position: usize, actor: Rc<dyn Player>) -> Seat {
        Seat {
            position,
            stack,
            stake: 0,
            status: BetStatus::Playing,
            hole: Hole::new(),
            actor,
        }
    }

    pub fn cards(&self) -> &Hole {
        &self.hole
    }

    pub fn valid_actions(&self, hand: &Hand) -> Vec<Action> {
        let mut actions = Vec::with_capacity(5);
        if self.can_check(hand) {
            actions.push(Action::Check(self.position));
        }
        if self.can_fold(hand) {
            actions.push(Action::Fold(self.position));
        }
        if self.can_call(hand) {
            actions.push(Action::Call(self.position, self.to_call(hand)));
        }
        if self.can_shove(hand) {
            actions.push(Action::Shove(self.position, self.to_shove(hand)));
        }
        if self.can_raise(hand) {
            actions.push(Action::Raise(self.position, self.min_raise(hand)));
        }
        actions
    }

    pub fn to_shove(&self, hand: &Hand) -> u32 {
        std::cmp::min(self.stack, hand.head.effective_stack() - self.stake)
    }
    pub fn to_call(&self, hand: &Hand) -> u32 {
        hand.head.effective_stake() - self.stake
    }
    pub fn min_raise(&self, hand: &Hand) -> u32 {
        (hand.min_raise() - self.stake)
    }
    pub fn max_raise(&self, hand: &Hand) -> u32 {
        self.to_shove(hand)
    }

    fn can_check(&self, hand: &Hand) -> bool {
        self.stake == hand.head.effective_stake()
    }
    fn can_shove(&self, hand: &Hand) -> bool {
        self.to_shove(hand) > 0
    }
    fn can_fold(&self, hand: &Hand) -> bool {
        self.to_call(hand) > 0
    }
    fn can_raise(&self, hand: &Hand) -> bool {
        self.to_shove(hand) > self.min_raise(hand)
    }
    fn can_call(&self, hand: &Hand) -> bool {
        self.can_fold(hand) && self.can_raise(hand)
    }
}
impl Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let card1 = self.hole.cards.get(0).unwrap();
        let card2 = self.hole.cards.get(1).unwrap();
        write!(
            f,
            "{:<3}{}   {}  {} {:>7}  ",
            self.position, self.status, card1, card2, self.stack,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BetStatus {
    Playing,
    Shoved,
    Folded,
}

impl Display for BetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BetStatus::Playing => write!(f, "P"),
            BetStatus::Shoved => write!(f, "S"),
            BetStatus::Folded => write!(f, "{}", "F".red()),
        }
    }
}

use super::{action::Action, hand::Hand, player::Player};
use crate::cards::hole::Hole;
use colored::Colorize;
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};
