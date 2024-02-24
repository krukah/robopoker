pub trait Actor {
    fn act(&self, game: &Game) -> Action;
}

#[derive(Debug, Clone)]
pub enum Action {
    Draw(Card),
    Check,
    Fold,
    Call(u32),
    Post(u32),
    Raise(u32),
    Shove(u32),
}
impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Action::Draw(card) => write!(f, "DRAW {}", card),
            Action::Check => write!(f, "CHECK"),
            Action::Fold => write!(f, "FOLD"),
            Action::Post(amount) => write!(f, "POST {}", amount),
            Action::Call(amount) => write!(f, "CALL {}", amount),
            Action::Raise(amount) => write!(f, "RAISE {}", amount),
            Action::Shove(amount) => write!(f, "SHOVE {}", amount),
        }
    }
}

use super::game::Game;
use crate::cards::card::Card;
use std::fmt::{Display, Formatter, Result};
