#![allow(dead_code)]

use crate::cards::card::Card;
use colored::*;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Draw(Card),
    Check(usize),
    Fold(usize),
    Call(usize, u32),
    Blind(usize, u32),
    Raise(usize, u32),
    Shove(usize, u32),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Action::Draw(card) => write!(f, "{}", format!("DEAL  {}", card).white()),
            Action::Check(id) => write!(f, "{id} {}", "CHECK".cyan()),
            Action::Fold(id) => write!(f, "{id} {}", "FOLD".red()),
            Action::Blind(id, amount) => write!(f, "{id} {}", format!("BLIND {}", amount).white()),
            Action::Call(id, amount) => write!(f, "{id} {}", format!("CALL  {}", amount).yellow()),
            Action::Raise(id, amount) => write!(f, "{id} {}", format!("RAISE {}", amount).green()),
            Action::Shove(id, amount) => {
                write!(f, "{id} {}", format!("SHOVE {}", amount).magenta())
            }
        }
    }
}
