#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Strength {
    HighCard(Rank),        // 4 kickers
    OnePair(Rank),         // 3 kickers
    TwoPair(Rank, Rank),   // 1 kickers
    ThreeOAK(Rank),        // 2 kickers
    Straight(Rank),        // 0 kickers
    Flush(Rank),           // 0 kickers
    FullHouse(Rank, Rank), // 0 kickers
    FourOAK(Rank),         // 1 kickers
    StraightFlush(Rank),   // 0 kickers
    MAX,
}

impl Ord for Strength {
    fn cmp(&self, other: &Self) -> Ordering {
        u8::from(self)
            .cmp(&u8::from(other))
            .then_with(|| match (self, other) {
                (Strength::StraightFlush(a), Strength::StraightFlush(b))
                | (Strength::FourOAK(a), Strength::FourOAK(b))
                | (Strength::Flush(a), Strength::Flush(b))
                | (Strength::Straight(a), Strength::Straight(b))
                | (Strength::ThreeOAK(a), Strength::ThreeOAK(b))
                | (Strength::OnePair(a), Strength::OnePair(b))
                | (Strength::HighCard(a), Strength::HighCard(b)) => a.cmp(b),

                (Strength::TwoPair(a, x), Strength::TwoPair(b, y))
                | (Strength::FullHouse(a, x), Strength::FullHouse(b, y)) => {
                    a.cmp(b).then_with(|| x.cmp(y))
                }

                _ => unreachable!(),
            })
    }
}

impl Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strength::HighCard(r) => write!(f, "HighCard      {}", r),
            Strength::OnePair(r) => write!(f, "OnePair       {}", r),
            Strength::TwoPair(r1, r2) => write!(f, "TwoPair       {}, {}", r1, r2),
            Strength::ThreeOAK(r) => write!(f, "ThreeOfAKind  {}", r),
            Strength::Straight(r) => write!(f, "Straight      {}", r),
            Strength::Flush(r) => write!(f, "Flush         {}", r),
            Strength::FullHouse(r1, r2) => write!(f, "FullHouse     {}, {}", r1, r2),
            Strength::FourOAK(r) => write!(f, "FourOfAKind   {}", r),
            Strength::StraightFlush(r) => write!(f, "StraightFlush {}", r),
            Strength::MAX => unreachable!(),
        }
    }
}

impl From<&Strength> for u8 {
    fn from(strength: &Strength) -> u8 {
        match strength {
            Strength::HighCard(_) => 0,
            Strength::OnePair(_) => 1,
            Strength::TwoPair(_, _) => 2,
            Strength::ThreeOAK(_) => 3,
            Strength::Straight(_) => 4,
            Strength::Flush(_) => 5,
            Strength::FullHouse(_, _) => 6,
            Strength::FourOAK(_) => 7,
            Strength::StraightFlush(_) => 8,
            Strength::MAX => u8::MAX,
        }
    }
}

impl PartialOrd for Strength {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use crate::cards::rank::Rank;
use std::cmp::Ordering;
use std::fmt::Display;
