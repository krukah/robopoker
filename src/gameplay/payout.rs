#[derive(Debug, Clone)]
pub struct Payout {
    pub position: usize,
    pub status: BetStatus,
    pub staked: u32,
    pub reward: u32,
    pub score: u32,
    pub rank: HandRank,
}

impl Display for Payout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.reward > 0 {
            write!(
                f,
                "{:<6} {}",
                format!("+{}", self.reward).green(),
                self.rank
            )
        } else {
            write!(f, "       {}", self.rank)
        }
    }
}

use super::seat::BetStatus;
use crate::evaluation::hand_rank::HandRank;
use colored::Colorize;
use std::fmt::Display;
