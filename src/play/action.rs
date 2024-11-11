use crate::cards::card::Card;
use crate::cards::hand::Hand;
use crate::Chips;
use colored::*;

const MASK: u32 = 0xFF;
const BITS: u32 = MASK.count_ones();

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Action {
    Draw(Hand),
    Fold,
    Call(Chips),
    Check,
    Raise(Chips),
    Shove(Chips),
    Blind(Chips),
}

impl Action {
    pub fn is_aggro(&self) -> bool {
        matches!(self, Action::Raise(_) | Action::Shove(_))
    }
    pub fn is_shove(&self) -> bool {
        matches!(self, Action::Shove(_))
    }
    pub fn is_raise(&self) -> bool {
        matches!(self, Action::Raise(_))
    }
    pub fn is_chance(&self) -> bool {
        matches!(self, Action::Draw(_))
    }
    pub fn is_choice(&self) -> bool {
        !self.is_chance()
    }
}

impl From<u32> for Action {
    fn from(value: u32) -> Self {
        let kind = value & MASK; // Use lowest 8 bits for action type
        let data = value >> BITS; // Shift right by 8 bits for data
        let bets = data as Chips;
        match kind {
            0 => Action::Fold,
            1 => Action::Check,
            2 => Action::Call(bets),
            3 => Action::Raise(bets),
            4 => Action::Shove(bets),
            5 => Action::Blind(bets),
            6 => Action::Draw(
                [0, 1, 2]
                    .iter()
                    .map(|i| BITS * i)
                    .map(|r| data >> r)
                    .map(|x| x & MASK)
                    .filter(|&x| x > 0)
                    .map(|x| x as u8 - 1)
                    .map(|c| Card::from(c))
                    .map(|c| Hand::from(c))
                    .fold(Hand::empty(), Hand::add),
            ),
            _ => panic!("at this disco"),
        }
    }
}

impl From<Action> for u32 {
    fn from(action: Action) -> Self {
        match action {
            Action::Fold => 0,
            Action::Check => 1,
            Action::Call(bets) => 2 | ((bets as u32) << BITS),
            Action::Raise(bets) => 3 | ((bets as u32) << BITS),
            Action::Shove(bets) => 4 | ((bets as u32) << BITS),
            Action::Blind(bets) => 5 | ((bets as u32) << BITS),
            Action::Draw(hand) => {
                6 | (hand
                    .into_iter()
                    .take(3)
                    .map(|c| u8::from(c))
                    .map(|c| c as u32 + 1)
                    .enumerate()
                    .map(|(i, x)| x << (i as u32 * BITS))
                    .fold(0u32, |hand, card| hand | card)
                    << BITS)
            }
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

    #[test]
    fn bijective_u32() {
        for action in [
            Action::Raise(1),
            Action::Blind(2),
            Action::Call(32767),
            Action::Shove(1738),
            Action::Draw(Hand::from("2c Th As")),
        ] {
            assert!(
                action == Action::from(u32::from(action)),
                "{}",
                format!("{} != {}", action, Action::from(u32::from(action))).red()
            );
        }
    }
}
