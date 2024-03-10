pub trait Player {
    fn id(&self) -> usize;
    fn act(&self, game: &Game) -> Action;
}

#[derive(Debug, Clone)]
pub enum Action {
    Draw(Card),
    Check(usize),
    Fold(usize),
    Call(usize, u32),
    Blind(usize, u32),
    Raise(usize, u32),
    Shove(usize, u32),
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Action::Draw(card) => write!(f, "{}", format!("DEAL  {}", card).white()),
            Action::Check(id) => write!(f, "{id} {}", "CHECK".cyan()),
            Action::Fold(id) => write!(f, "{id} {}", "FOLD".red()),
            Action::Blind(id, amount) => write!(f, "{id} {}", format!("BLIND {}", amount).white()),
            Action::Call(id, amount) => write!(f, "{id} {}", format!("CALL  {}", amount).cyan()),
            Action::Raise(id, amount) => write!(f, "{id} {}", format!("RAISE {}", amount).green()),
            Action::Shove(id, amount) => write!(f, "{id} {}", format!("SHOVE {}", amount).red()),
        }
    }
}

use super::game::Game;
use crate::cards::card::Card;
use colored::*;
use std::fmt::{Display, Formatter, Result};
