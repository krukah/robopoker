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

impl From<u32> for Action {
    fn from(value: u32) -> Self {
        let kind = value & MASK; // Use lowest 8 bits for action type
        let data = value >> BITS; // Shift right by 8 bits for data
        match kind {
            0 => Action::Fold,
            1 => Action::Check,
            2 => Action::Call(data as Chips),
            3 => Action::Raise(data as Chips),
            4 => Action::Shove(data as Chips),
            5 => Action::Blind(data as Chips),
            6 => {
                let hand = [0, 1, 2]
                    .iter()
                    .map(|i| ((data >> (BITS * i)) & MASK) as u8)
                    .filter(|&c| c > 0)
                    .map(|c| Hand::from(Card::from(c)))
                    .fold(Hand::empty(), Hand::add);
                Action::Draw(hand)
            }
            _ => panic!("Invalid action value"),
        }
    }
}

impl From<Action> for u32 {
    fn from(action: Action) -> Self {
        match action {
            Action::Fold => 0,
            Action::Check => 1,
            Action::Call(amount) => 2 | ((amount as u32) << BITS),
            Action::Raise(amount) => 3 | ((amount as u32) << BITS),
            Action::Shove(amount) => 4 | ((amount as u32) << BITS),
            Action::Blind(amount) => 5 | ((amount as u32) << BITS),
            Action::Draw(hand) => {
                let data = hand
                    .into_iter()
                    .take(3)
                    .enumerate()
                    .map(|(i, c)| (u8::from(c) as u32) << (i * BITS as usize))
                    .fold(0u32, |hand, card| hand | card);
                6 | (data << BITS)
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
            Action::Draw(Hand::from("2d 3d 4d")),
        ] {
            assert!(action == Action::from(u32::from(action)));
        }
    }
}
