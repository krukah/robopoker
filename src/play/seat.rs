use super::action::Action;
use super::game::Game;
use super::Chips;
use crate::cards::hole::Hole;
use colored::Colorize;

#[derive(Debug, Clone, Copy)]
pub struct Seat {
    cards: Hole,
    stack: Chips,
    stake: Chips,
    status: Status,
}

impl Seat {
    pub fn new(stack: Chips) -> Seat {
        Seat {
            stack,
            stake: 0,
            status: Status::Playing,
            cards: Hole::new(),
        }
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
    pub fn hole_ref(&self) -> &Hole {
        &self.cards
    }
    pub fn hole_mut(&mut self) -> &mut Hole {
        &mut self.cards
    }
    pub fn act(&self, _: &Game) -> Action {
        todo!()
    }

    pub fn bet(&mut self, bet: &Chips) {
        self.stack -= bet;
        self.stake += bet;
    }
    pub fn win(&mut self, winnings: &Chips) {
        println!("{}{}", self, winnings);
        self.stack += winnings;
    }
    pub fn set_sttus(&mut self, status: Status) {
        self.status = status;
    }
    pub fn set_cards(&mut self, cards: Hole) {
        self.cards = cards;
    }
    pub fn set_stake(&mut self) {
        self.stake = 0;
    }
}
impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
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
            Status::Playing => write!(f, "{}", "P".green()),
            Status::Shoving => write!(f, "{}", "S".yellow()),
            Status::Folding => write!(f, "{}", "F".red()),
        }
    }
}
