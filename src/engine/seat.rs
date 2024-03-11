#[derive(Debug, Clone)]
pub struct Seat {
    pub id: usize,
    pub stake: u32,
    pub stack: u32,
    pub status: BetStatus,
    pub player: Player,
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
            BetStatus::Folded => write!(f, "F"),
        }
    }
}

impl Seat {
    pub fn new(stack: u32, position: usize) -> Seat {
        Seat {
            id: position,
            player: Player::Robot(Hole::new()),
            stack,
            stake: 0,
            status: BetStatus::Playing,
        }
    }

    pub fn valid_actions(&self, hand: &Hand) -> Vec<Action> {
        let mut actions = Vec::with_capacity(5);
        if self.can_check(hand) {
            actions.push(Action::Check(self.id));
        }
        if self.can_fold(hand) {
            actions.push(Action::Fold(self.id));
        }
        if self.can_call(hand) {
            actions.push(Action::Call(self.id, self.to_call(hand)));
        }
        if self.can_shove(hand) {
            actions.push(Action::Shove(self.id, self.to_shove(hand)));
        }
        if self.can_raise(hand) {
            actions.push(Action::Raise(self.id, self.to_raise(hand)));
        }
        actions
    }

    pub fn to_call(&self, hand: &Hand) -> u32 {
        hand.head.table_stake() - self.stake
    }
    pub fn to_shove(&self, hand: &Hand) -> u32 {
        std::cmp::min(self.stack, hand.head.table_stack() - self.stake)
    }
    pub fn to_raise(&self, hand: &Hand) -> u32 {
        std::cmp::min(self.to_shove(hand) - 1, 5)
    }

    fn can_check(&self, hand: &Hand) -> bool {
        self.stake == hand.head.table_stake()
    }
    fn can_shove(&self, hand: &Hand) -> bool {
        self.to_shove(hand) > 0
    }
    fn can_fold(&self, hand: &Hand) -> bool {
        self.to_call(hand) > 0
    }
    fn can_raise(&self, hand: &Hand) -> bool {
        self.to_shove(hand) > self.to_call(hand) + 1
    }
    fn can_call(&self, hand: &Hand) -> bool {
        self.can_fold(hand) && self.can_raise(hand)
    }
}
impl Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (card1, card2) = match &self.player {
            Player::Human(hole) | Player::Robot(hole) => {
                (hole.cards.get(0).unwrap(), hole.cards.get(1).unwrap())
            }
        };
        write!(
            f,
            "{:<3}{}   {}  {} {:>7}  \n",
            self.id, self.status, card1, card2, self.stack,
        )
    }
}

use super::{action::Action, game::Hand, player::Player};
use crate::cards::hole::Hole;
use std::fmt::{Debug, Display};
