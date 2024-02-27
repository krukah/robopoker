pub trait Player {
    fn act(&self, game: &Game) -> Action;
}

#[derive(Debug, Clone)]
pub enum Action {
    Draw(Card),
    Check,
    Fold,
    Call(u32),
    Blind(u32),
    Raise(u32),
    Shove(u32),
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Action::Draw(card) => write!(f, "{}", format!("DEAL  {}", card).white()),
            Action::Check => write!(f, "{}", "CHECK".cyan()),
            Action::Fold => write!(f, "{}", "FOLD".red()),
            Action::Blind(amount) => write!(f, "{}", format!("BLIND {}", amount).white()),
            Action::Call(amount) => write!(f, "{}", format!("CALL  {}", amount).cyan()),
            Action::Raise(amount) => write!(f, "{}", format!("RAISE {}", amount).green()),
            Action::Shove(amount) => write!(f, "{}", format!("SHOVE {}", amount).red()),
        }
    }
}

use super::game::Game;
use crate::cards::card::Card;
use colored::*;
use std::fmt::{Display, Formatter, Result};
