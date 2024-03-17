#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum HandRank {
    HighCard(Rank),
    OnePair(Rank),
    TwoPair(Rank, Rank),
    ThreeOfAKind(Rank),
    Straight(Rank),
    Flush(Rank),
    FullHouse(Rank, Rank),
    FourOfAKind(Rank),
    StraightFlush(Rank),
}
impl Display for HandRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandRank::HighCard(r) => write!(f, "HighCard      {}", r),
            HandRank::OnePair(r) => write!(f, "OnePair       {}", r),
            HandRank::TwoPair(r1, r2) => write!(f, "TwoPair       {}, {}", r1, r2),
            HandRank::ThreeOfAKind(r) => write!(f, "ThreeOfAKind  {}", r),
            HandRank::Straight(r) => write!(f, "Straight      {}", r),
            HandRank::Flush(r) => write!(f, "Flush         {}", r),
            HandRank::FullHouse(r1, r2) => write!(f, "FullHouse     {}, {}", r1, r2),
            HandRank::FourOfAKind(r) => write!(f, "FourOfAKind   {}", r),
            HandRank::StraightFlush(r) => write!(f, "StraightFlush {}", r),
        }
    }
}

// impl Ord for HandRank {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match (self, other) {
//             (HandRank::HighCard(a), HandRank::HighCard(b)) => a.cmp(b),
//             (HandRank::OnePair(a), HandRank::OnePair(b)) => a.cmp(b),
//             (HandRank::TwoPair(a1, a2), HandRank::TwoPair(b1, b2)) => (a1, a2).cmp(&(b1, b2)),
//             (HandRank::ThreeOfAKind(a), HandRank::ThreeOfAKind(b)) => a.cmp(b),
//             (HandRank::Straight(a), HandRank::Straight(b)) => a.cmp(b),
//             (HandRank::Flush(a), HandRank::Flush(b)) => a.cmp(b),
//             (HandRank::FullHouse(a1, a2), HandRank::FullHouse(b1, b2)) => (a1, a2).cmp(&(b1, b2)),
//             (HandRank::FourOfAKind(a), HandRank::FourOfAKind(b)) => a.cmp(b),
//             (HandRank::StraightFlush(a), HandRank::StraightFlush(b)) => a.cmp(b),
//             (a, b) => (a as *const HandRank).cmp(&(b as *const HandRank)),
//         }
//     }
// }

// impl PartialOrd for HandRank {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

use std::fmt::Display;

use crate::cards::rank::Rank;
// use std::cmp::Ordering;
