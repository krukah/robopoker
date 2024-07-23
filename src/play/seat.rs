use super::{action::Action, game::Game};
use crate::cards::hole::Hole;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct Seat {
    position: usize,
    hole: Hole,
    stack: u32,
    stake: u32,
    status: BetStatus,
}

impl Seat {
    pub fn new(stack: u32, position: usize) -> Seat {
        Seat {
            position,
            stack,
            stake: 0,
            status: BetStatus::Playing,
            hole: Hole::new(),
        }
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn stack(&self) -> u32 {
        self.stack
    }
    pub fn stake(&self) -> u32 {
        self.stake
    }
    pub fn status(&self) -> BetStatus {
        self.status
    }
    pub fn peek(&self) -> &Hole {
        &self.hole
    }
    pub fn hole(&mut self) -> &mut Hole {
        &mut self.hole
    }
    pub fn act(&self, _hand: &Game) -> Action {
        todo!()
    }

    pub fn bet(&mut self, bet: u32) {
        self.stack -= bet;
        self.stake += bet;
    }
    pub fn win(&mut self, winnings: u32) {
        println!("{}{}", self, winnings);
        self.stack += winnings;
    }
    pub fn set(&mut self, status: BetStatus) {
        self.status = status;
    }
    pub fn clear(&mut self) {
        self.stake = 0;
    }
    pub fn assign(&mut self, position: usize) {
        self.position = position;
    }

    pub fn valid_actions(&self, hand: &Game) -> Vec<Action> {
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

    pub fn to_shove(&self, hand: &Game) -> u32 {
        std::cmp::min(self.stack, hand.head.effective_stack() - self.stake)
    }
    pub fn to_call(&self, hand: &Game) -> u32 {
        hand.head.effective_stake() - self.stake
    }
    pub fn min_raise(&self, hand: &Game) -> u32 {
        hand.min_raise() - self.stake
    }
    pub fn max_raise(&self, hand: &Game) -> u32 {
        self.to_shove(hand)
    }

    fn can_check(&self, hand: &Game) -> bool {
        self.stake == hand.head.effective_stake()
    }
    fn can_shove(&self, hand: &Game) -> bool {
        self.to_shove(hand) > 0
    }
    fn can_fold(&self, hand: &Game) -> bool {
        self.to_call(hand) > 0
    }
    fn can_raise(&self, hand: &Game) -> bool {
        self.to_shove(hand) >= self.min_raise(hand)
    }
    fn can_call(&self, hand: &Game) -> bool {
        self.can_fold(hand) && self.can_raise(hand)
    }
}
impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!("just write status and hole")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BetStatus {
    Playing,
    Shoved,
    Folded,
}

impl std::fmt::Display for BetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BetStatus::Playing => write!(f, "P"),
            BetStatus::Shoved => write!(f, "S"),
            BetStatus::Folded => write!(f, "{}", "F".red()),
        }
    }
}
