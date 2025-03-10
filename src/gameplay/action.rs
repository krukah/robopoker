use crate::cards::card::Card;
use crate::cards::hand::Hand;
use crate::Chips;

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
    pub fn is_choice(&self) -> bool {
        !self.is_chance()
    }
    pub fn is_chance(&self) -> bool {
        matches!(self, Action::Draw(_))
    }
    pub fn is_aggro(&self) -> bool {
        matches!(self, Action::Raise(_) | Action::Shove(_))
    }
    pub fn is_shove(&self) -> bool {
        matches!(self, Action::Shove(_))
    }
    pub fn is_raise(&self) -> bool {
        matches!(self, Action::Raise(_))
    }
    pub fn is_blind(&self) -> bool {
        matches!(self, Action::Blind(_))
    }
}
impl From<Action> for String {
    fn from(action: Action) -> Self {
        match action {
            Action::Fold => format!("FOLD"),
            Action::Check => format!("CHECK"),
            Action::Draw(card) => format!("DEAL  {}", card),
            Action::Call(amount) => format!("CALL  {}", amount),
            Action::Blind(amount) => format!("BLIND {}", amount),
            Action::Raise(amount) => format!("RAISE {}", amount),
            Action::Shove(amount) => format!("SHOVE {}", amount),
        }
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
            _ => panic!("at the disco"),
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
impl TryFrom<&str> for Action {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts[0].to_uppercase().as_str() {
            "CHECK" => Ok(Action::Check),
            "FOLD" => Ok(Action::Fold),
            "CALL" => parts
                .get(1)
                .and_then(|n| n.parse().ok())
                .map(Action::Call)
                .ok_or("invalid call amount"),
            "RAISE" => parts
                .get(1)
                .and_then(|n| n.parse().ok())
                .map(Action::Raise)
                .ok_or("invalid raise amount"),
            "SHOVE" => parts
                .get(1)
                .and_then(|n| n.parse().ok())
                .map(Action::Shove)
                .ok_or("invalid shove amount"),
            "BLIND" => parts
                .get(1)
                .and_then(|n| n.parse().ok())
                .map(Action::Blind)
                .ok_or("invalid blind amount"),
            "DEAL" => Hand::try_from(parts[1..].join(" ").as_str())
                .map(Action::Draw)
                .map_err(|_| "invalid deal cards"),
            _ => Err("invalid action type"),
        }
    }
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        String::from(*self).fmt(f)
    }
}

#[cfg(feature = "native")]
impl From<Action> for colored::Color {
    fn from(action: Action) -> Self {
        match action {
            Action::Fold => colored::Color::Red,
            Action::Check => colored::Color::Yellow,
            Action::Call(_) => colored::Color::Green,
            Action::Raise(_) => colored::Color::Green,
            Action::Shove(_) => colored::Color::Green,
            Action::Blind(_) => colored::Color::White,
            Action::Draw(_) => colored::Color::White,
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
            Action::Draw(Hand::try_from("2c Th As").unwrap()),
        ] {
            assert!(action == Action::from(u32::from(action)));
        }
    }
}
