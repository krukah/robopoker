use super::{showdown::Showdown, Chips};
use crate::cards::strength::Strength;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct Payout {
    reward: Chips,
    strength: Strength,
}

impl std::fmt::Display for Payout {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.reward > 0 {
            let reward = format!("+{}", self.reward).green();
            let strength = self.strength.to_string();
            write!(f, "{:<6}{}", reward, strength)
        } else {
            write!(f, "      {}", self.strength)
        }
    }
}

impl From<Showdown> for [Payout; 10] {
    fn from(showdown: Showdown) -> Self {
        todo!()
    }
}
