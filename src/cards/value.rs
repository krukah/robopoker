use super::rank::Rank;

/// A poker hand's value.
///
/// This is a simplified version of the hand's value, and does not include the hand's kicker cards.
/// The value is ordered by the hand's Strength, and the kicker cards are used to break ties.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Value {
    HighCard(Rank),        // 4 kickers
    OnePair(Rank),         // 3 kickers
    TwoPair(Rank, Rank),   // 1 kickers
    ThreeOAK(Rank),        // 2 kickers
    Straight(Rank),        // 0 kickers
    Flush(Rank),           // 0 kickers
    FullHouse(Rank, Rank), // 0 kickers
    FourOAK(Rank),         // 1 kickers
    StraightFlush(Rank),   // 0 kickers
    MAX,                   // useful for showdown implementation
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::MAX => unreachable!(),
            Value::FullHouse(r1, r2) => write!(f, "FullHouse     {}{}", r1, r2),
            Value::TwoPair(r1, r2) => write!(f, "TwoPair       {}{}", r1, r2),
            Value::HighCard(r) => write!(f, "HighCard      {} ", r),
            Value::OnePair(r) => write!(f, "OnePair       {} ", r),
            Value::ThreeOAK(r) => write!(f, "ThreeOfAKind  {} ", r),
            Value::Straight(r) => write!(f, "Straight      {} ", r),
            Value::FourOAK(r) => write!(f, "FourOfAKind   {} ", r),
            Value::Flush(r) => write!(f, "Flush         {} ", r),
            Value::StraightFlush(r) => write!(f, "StraightFlush {} ", r),
        }
    }
}
