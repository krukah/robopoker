use super::evaluator::Evaluator;
use super::hand::Hand;
use super::kicks::Kicks;
use super::value::Value;

/// A hand's strength.
///
/// This will always be constructed from a Hand, which is an unordered
/// set of Cards. The strength is determined by the Hand's value, and the
/// kicker cards are used to break ties.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Strength {
    value: Value,
    kicks: Kicks,
}

impl From<Hand> for Strength {
    fn from(hand: Hand) -> Self {
        Self::from(Evaluator::from(hand))
    }
}

impl From<(Value, Kicks)> for Strength {
    fn from((value, kicks): (Value, Kicks)) -> Self {
        Self { value, kicks }
    }
}

impl std::fmt::Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:<18}", self.value)
    }
}
