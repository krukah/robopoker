use super::hand::Hand;

/// A hand's kicker cards.
///
/// This is a simplified version of the hand's value, and does not include the hand's kicker cards.
/// The value is ordered by the hand's strength, and the kicker cards are used to break ties.
/// WARNING: Implementation of Ord will not correctly compare Suits.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Kicks(Hand);

/// Kicker isomorphism
/// structurally identifcal, semantically different from Hand
impl From<Hand> for Kicks {
    fn from(hand: Hand) -> Self {
        Self(hand)
    }
}
impl From<Kicks> for Hand {
    fn from(k: Kicks) -> Self {
        k.0
    }
}

impl std::fmt::Display for Kicks {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
