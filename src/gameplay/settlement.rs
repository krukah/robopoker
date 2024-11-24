use crate::cards::strength::Strength;
use crate::gameplay::seat::State;
use crate::Chips;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct Settlement {
    pub reward: Chips,
    pub risked: Chips,
    pub status: State,
    pub strength: Strength,
}

impl Settlement {
    pub fn pnl(&self) -> Chips {
        self.reward - self.risked
    }
}

impl From<(Chips, State, Strength)> for Settlement {
    fn from((risked, status, strength): (Chips, State, Strength)) -> Self {
        Self {
            reward: 0,
            risked,
            status,
            strength,
        }
    }
}

impl std::fmt::Display for Settlement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.reward > 0 {
            let reward = format!("+{}", self.reward).green();
            write!(f, "{:<5}{}", reward, self.strength)
        } else {
            write!(f, "     {}", self.strength)
        }
    }
}
