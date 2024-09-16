use super::action::Action;
use super::game::Game;
use super::showdown::Showdown;
use super::Chips;
use crate::cards::hole::Hole;
use colored::Colorize;

#[derive(Debug, Clone, Copy)]
pub struct Seat {
    cards: Hole,
    stack: Chips,
    stake: Chips,
    state: State,
}

impl Seat {
    pub fn new(stack: Chips) -> Seat {
        Seat {
            stack,
            stake: 0,
            state: State::Playing,
            cards: Hole::new(),
        }
    }
    pub fn stack(&self) -> Chips {
        self.stack
    }
    pub fn stake(&self) -> Chips {
        self.stake
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn cards(&self) -> &Hole {
        &self.cards
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
    pub fn set_state(&mut self, status: State) {
        self.state = status;
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
            self.state,
            self.cards
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Playing,
    Shoving,
    Folding,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Playing => write!(f, "{}", "P".green()),
            State::Shoving => write!(f, "{}", "S".yellow()),
            State::Folding => write!(f, "{}", "F".red()),
        }
    }
}
