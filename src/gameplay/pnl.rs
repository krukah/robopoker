use super::*;
use crate::Chips;

/// Public profit/loss information visible to all players.
/// Represents chip movements and player state without revealing hole cards.
#[derive(Debug, Clone)]
pub struct PnL {
    reward: Chips,
    risked: Chips,
    status: State,
}

impl PnL {
    pub fn new(reward: Chips, risked: Chips, status: State) -> Self {
        Self {
            reward,
            risked,
            status,
        }
    }
    pub fn add(&mut self, amount: Chips) {
        self.reward += amount;
    }
    pub fn won(&self) -> Chips {
        self.reward() - self.risked()
    }
    pub fn reward(&self) -> Chips {
        self.reward
    }
    pub fn risked(&self) -> Chips {
        self.risked
    }
    pub fn status(&self) -> State {
        self.status
    }
}

impl std::fmt::Display for PnL {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:+}", self.won())
    }
}
