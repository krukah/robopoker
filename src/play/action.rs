#![allow(dead_code)]

use super::Chips;
use crate::cards::card::Card;
use colored::*;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Draw(Card),
    Check(usize),
    Fold(usize),
    Call(usize, Chips),
    Blind(usize, Chips),
    Raise(usize, Chips),
    Shove(usize, Chips),
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
