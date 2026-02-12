use super::*;
use rbp_core::Chips;

/// Public profit/loss information.
///
/// Tracks chips risked and won without revealing hole cards. This is the
/// information visible to all players at showdown before cards are revealed.
///
/// # Fields
///
/// - `reward` — Total chips received from pot
/// - `risked` — Total chips committed to pot
/// - `status` — Final betting state (for determining eligibility)
#[derive(Debug, Clone)]
pub struct PnL {
    reward: Chips,
    risked: Chips,
    status: State,
}

impl PnL {
    /// Creates a PnL with initial values.
    pub fn new(reward: Chips, risked: Chips, status: State) -> Self {
        Self {
            reward,
            risked,
            status,
        }
    }
    /// Adds chips to reward.
    pub fn add(&mut self, amount: Chips) {
        self.reward += amount;
    }
    /// Net profit (can be negative for losses).
    pub fn won(&self) -> Chips {
        self.reward() - self.risked()
    }
    /// Total chips received from pot.
    pub fn reward(&self) -> Chips {
        self.reward
    }
    /// Total chips committed to pot.
    pub fn risked(&self) -> Chips {
        self.risked
    }
    /// Final betting state.
    pub fn status(&self) -> State {
        self.status
    }
}

impl std::fmt::Display for PnL {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:+}", self.won())
    }
}
