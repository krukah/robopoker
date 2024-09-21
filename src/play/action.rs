#![allow(dead_code)]

use super::Chips;
use crate::cards::card::Card;
use colored::*;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Action {
    Draw(Card),
    Blind(Chips),
    Shove(Chips),
    Raise(Chips),
    Call(Chips),
    Check,
    Fold,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Action::Draw(card) => write!(f, "{}", format!("DEAL  {}", card).white()),
            Action::Check => write!(f, "{}", "CHECK".cyan()),
            Action::Fold => write!(f, "{}", "FOLD".red()),
            Action::Blind(amount) => write!(f, "{}", format!("BLIND {}", amount).white()),
            Action::Call(amount) => write!(f, "{}", format!("CALL  {}", amount).yellow()),
            Action::Raise(amount) => write!(f, "{}", format!("RAISE {}", amount).green()),
            Action::Shove(amount) => write!(f, "{}", format!("SHOVE {}", amount).magenta()),
        }
    }
}
