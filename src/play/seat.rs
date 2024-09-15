use super::action::Action;
use super::game::Game;
use super::Chips;
use crate::cards::hole::Hole;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct Seat {
    cards: Hole,
    stack: Chips,
    stake: Chips,
    status: Status,
    position: usize, // removed
}

impl Seat {
    pub fn new(stack: Chips, position: usize) -> Seat {
        Seat {
            position,
            stack,
            stake: 0,
            status: Status::Playing,
            cards: Hole::new(),
        }
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn stack(&self) -> Chips {
        self.stack
    }
    pub fn stake(&self) -> Chips {
        self.stake
    }
    pub fn status(&self) -> Status {
        self.status
    }
    pub fn peek(&self) -> &Hole {
        &self.cards
    }
    pub fn hole(&mut self) -> &mut Hole {
        &mut self.cards
    }
    pub fn act(&self, _hand: &Game) -> Action {
        todo!()
    }

    pub fn bet(&mut self, bet: Chips) {
        self.stack -= bet;
        self.stake += bet;
    }
    pub fn win(&mut self, winnings: Chips) {
        println!("{}{}", self, winnings);
        self.stack += winnings;
    }
    pub fn set(&mut self, status: Status) {
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

    pub fn to_shove(&self, hand: &Game) -> Chips {
        std::cmp::min(self.stack, hand.head.effective_stack() - self.stake)
    }
    pub fn to_call(&self, hand: &Game) -> Chips {
        hand.head.effective_stake() - self.stake
    }
    pub fn min_raise(&self, hand: &Game) -> Chips {
        hand.min_raise() - self.stake
    }
    pub fn max_raise(&self, hand: &Game) -> Chips {
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
        write!(
            f,
            "{}{}{}{}{}",
            format!("{:02}", self.position).cyan(),
            format!("{:04}", self.stack).green(),
            format!("{:04}", self.stake).yellow(),
            self.status,
            self.cards
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Playing,
    Shoving,
    Folding,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Playing => write!(f, "P"),
            Status::Shoving => write!(f, "S"),
            Status::Folding => write!(f, "{}", "F".red()),
        }
    }
}
