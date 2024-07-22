#[derive(Debug, Clone)]
pub struct Payout {
    pub position: usize,
    pub strength: Strength,
    pub status: BetStatus,
    pub risked: u32,
    pub reward: u32,
}

impl std::fmt::Display for Payout {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.reward > 0 {
            write!(
                f,
                "{:<6}{}",
                format!("+{}", self.reward).green(),
                self.strength
            )
        } else {
            write!(f, "      {}", self.strength)
        }
    }
}

use super::seat::BetStatus;
use crate::cards::strength::Strength;
use colored::Colorize;
