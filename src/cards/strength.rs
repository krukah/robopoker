use super::evaluator::Evaluator;
use super::hand::Hand;
use super::kicks::Kickers;
use super::value::Ranking;

/// A hand's strength.
///
/// This will always be constructed from a Hand, which is an unordered
/// set of Cards. The strength is determined by the Hand's value, and the
/// kicker cards are used to break ties.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Strength {
    value: Ranking,
    kicks: Kickers,
}

impl From<Hand> for Strength {
    fn from(hand: Hand) -> Self {
        Self::from(Evaluator::from(hand))
    }
}

impl From<(Ranking, Kickers)> for Strength {
    fn from((value, kicks): (Ranking, Kickers)) -> Self {
        Self { value, kicks }
    }
}

impl std::fmt::Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:<18}", self.value)
    }
}
