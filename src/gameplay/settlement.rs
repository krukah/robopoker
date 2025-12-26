use super::*;
use crate::Chips;
use crate::cards::*;

/// Complete settlement including private hand strength.
/// Used at showdown when hole cards are revealed.
#[derive(Debug, Clone)]
pub struct Settlement {
    revealed: PnL,
    strength: Strength,
}

impl Settlement {
    pub fn new(revealed: PnL, strength: Strength) -> Self {
        Self { revealed, strength }
    }
    pub fn pnl(&self) -> &PnL {
        &self.revealed
    }
    pub fn strength(&self) -> &Strength {
        &self.strength
    }
    pub fn won(&self) -> Chips {
        self.pnl().won()
    }
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
