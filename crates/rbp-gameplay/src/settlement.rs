use super::*;
use rbp_core::Chips;
use rbp_cards::*;

/// A player's final result including hand strength.
///
/// Combines the public [`PnL`] (chips risked/won) with the private
/// [`Strength`] (hand ranking). Used by [`Showdown`] to determine
/// pot distributions.
#[derive(Debug, Clone)]
pub struct Settlement {
    revealed: PnL,
    strength: Strength,
}

impl Settlement {
    /// Creates a settlement from profit/loss info and hand strength.
    pub fn new(revealed: PnL, strength: Strength) -> Self {
        Self { revealed, strength }
    }
    /// Public profit/loss information.
    pub fn pnl(&self) -> &PnL {
        &self.revealed
    }
    /// Hand strength for showdown comparison.
    pub fn strength(&self) -> &Strength {
        &self.strength
    }
    /// Net chips won (reward - risked).
    pub fn won(&self) -> Chips {
        self.pnl().won()
    }
    /// Adds chips to the reward (for pot distribution).
    pub fn add(&mut self, amount: Chips) {
        self.revealed.add(amount);
    }
}

impl From<(Chips, State, Strength)> for Settlement {
    fn from((risked, status, strength): (Chips, State, Strength)) -> Self {
        Self::new(PnL::new(0, risked, status), strength)
    }
}

impl std::fmt::Display for Settlement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let strength = self.strength();
        let pnl = self.pnl().reward();
        if pnl > 0 {
            write!(f, "{:<5}{}", format!("+{}", pnl), strength)
        } else {
            write!(f, "     {}", strength)
        }
    }
}
