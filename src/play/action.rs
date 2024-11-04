use crate::cards::card::Card;
use crate::Chips;
use colored::*;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Action {
    Draw(Card),
    Fold,
    Call(Chips),
    Check,
    Raise(Chips),
    Shove(Chips),
    Blind(Chips),
}

impl From<u32> for Action {
    fn from(value: u32) -> Self {
        let action_type = value & 0xFFFF;
        let data = (value >> 16) as Chips;
        match action_type {
            0 => Action::Fold,
            1 => Action::Check,
            2 => Action::Call(data),
            3 => Action::Raise(data),
            4 => Action::Shove(data),
            5 => Action::Blind(data),
            6 => Action::Draw(Card::from(data as u8)),
            _ => panic!("Invalid action value"),
        }
    }
}

impl From<Action> for u32 {
    fn from(action: Action) -> Self {
        match action {
            Action::Fold => 0,
            Action::Check => 1,
            Action::Call(amount) => 2 | ((amount as u32) << 16),
            Action::Raise(amount) => 3 | ((amount as u32) << 16),
            Action::Shove(amount) => 4 | ((amount as u32) << 16),
            Action::Blind(amount) => 5 | ((amount as u32) << 16),
            Action::Draw(card) => 6 | ((u8::from(card) as u32) << 16),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card::Card;

    #[test]
    fn bijective_u32() {
        assert!([
            Action::Raise(1),
            Action::Blind(2),
            Action::Call(32767),
            Action::Shove(1738),
            Action::Draw(Card::from(51u8)),
        ]
        .into_iter()
        .all(|action| action == Action::from(u32::from(action))));
    }
}
