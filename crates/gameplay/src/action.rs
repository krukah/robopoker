use rbp_cards::*;
use rbp_core::*;

const MASK: u32 = 0xFF;
const BITS: u32 = MASK.count_ones();

/// A player decision or chance event in the game.
///
/// Actions represent the atomic transitions in poker: betting decisions made
/// by players (fold, check, call, raise, shove) and chance events (card deals,
/// blind posts). Each variant carries the relevant chip amount or cards.
///
/// # Serialization
///
/// Actions pack into `u32` for compact storage: 8 bits for the variant tag,
/// remaining bits for chip amounts or card data.
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
    /// True if this is a player decision (not a card deal).
    pub fn is_choice(&self) -> bool {
        !self.is_chance()
    }
    /// True if this is a card deal (chance node).
    pub fn is_chance(&self) -> bool {
        matches!(self, Action::Draw(_))
    }
    /// True if this is a raise or shove (aggressive action).
    pub fn is_aggro(&self) -> bool {
        matches!(self, Action::Raise(_) | Action::Shove(_))
    }
    /// True if this is an all-in bet.
    pub fn is_shove(&self) -> bool {
        matches!(self, Action::Shove(_))
    }
    /// True if this is a raise (not all-in).
    pub fn is_raise(&self) -> bool {
        matches!(self, Action::Raise(_))
    }
    /// True if this is a blind post.
    pub fn is_blind(&self) -> bool {
        matches!(self, Action::Blind(_))
    }
    /// True if this is a fold or check (no chips added).
    pub fn is_passive(&self) -> bool {
        matches!(self, Action::Fold | Action::Check)
    }
}

impl Action {
    /// Extracts the dealt cards from a Draw action.
    pub fn hand(&self) -> Option<Hand> {
        if let Action::Draw(hand) = self {
            Some(hand.clone())
        } else {
            None
        }
    }
    /// Extracts the chip amount from betting actions.
    pub fn amount(&self) -> Option<Chips> {
        match *self {
            Action::Call(amount)
            | Action::Raise(amount)
            | Action::Shove(amount)
            | Action::Blind(amount) => Some(amount),
            _ => None,
        }
    }
    /// Compact symbol for path serialization (e.g., "C100", "R50").
    pub fn symbol(&self) -> String {
        match self {
            Action::Fold => format!("F"),
            Action::Check => format!("X"),
            Action::Draw(h) => format!("{}", h),
            Action::Call(n) => format!("C{}", n),
            Action::Blind(n) => format!("B{}", n),
            Action::Raise(n) => format!("R{}", n),
            Action::Shove(n) => format!("S{}", n),
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Action::Fold => "Fold",
            Action::Check => "Check",
            Action::Call(_) => "Call",
            Action::Raise(_) => "Raise",
            Action::Shove(_) => "Shove",
            Action::Draw(_) => "Draw",
            Action::Blind(_) => "Blind",
        }
    }
    pub fn abbrev(&self) -> &'static str {
        match self {
            Action::Fold => "-",
            Action::Check => "•",
            Action::Call(_) => "=",
            Action::Raise(_) => "+",
            Action::Shove(_) => "!",
            Action::Draw(_) => "?",
            Action::Blind(_) => "$",
        }
    }
}

impl From<Action> for String {
    fn from(action: Action) -> Self {
        action.to_string()
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
        match self {
            Action::Fold => write!(f, "FOLD"),
            Action::Check => write!(f, "CHECK"),
            Action::Draw(hand) => write!(f, "DEAL  {}", hand),
            Action::Call(amount) => write!(f, "CALL  {}", amount),
            Action::Blind(amount) => write!(f, "BLIND {}", amount),
            Action::Raise(amount) => write!(f, "RAISE {}", amount),
            Action::Shove(amount) => write!(f, "SHOVE {}", amount),
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
