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
    MUCK,
    MAX,
}

impl Ord for Strength {
    fn cmp(&self, other: &Self) -> Ordering {
        match u8::from(self).cmp(&u8::from(other)) {
            Ordering::Equal => match (self, other) {
                (Strength::TwoPair(a1, a2), Strength::TwoPair(b1, b2))
                | (Strength::FullHouse(a1, a2), Strength::FullHouse(b1, b2)) => match a1.cmp(a2) {
                    Ordering::Equal => b1.cmp(b2),
                    x => x,
                },

                (Strength::StraightFlush(a), Strength::StraightFlush(b))
                | (Strength::Straight(a), Strength::Straight(b))
                | (Strength::ThreeOAK(a), Strength::ThreeOAK(b))
                | (Strength::HighCard(a), Strength::HighCard(b))
                | (Strength::FourOAK(a), Strength::FourOAK(b))
                | (Strength::OnePair(a), Strength::OnePair(b))
                | (Strength::Flush(a), Strength::Flush(b)) => a.cmp(b),

                _ => unreachable!(),
            },
            x => return x,
        }
    }
}

impl Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strength::MUCK => write!(f, ""),
            Strength::MAX => unreachable!(),
            Strength::HighCard(r) => write!(f, "HighCard      {}", r),
            Strength::OnePair(r) => write!(f, "OnePair       {}", r),
            Strength::TwoPair(r1, r2) => write!(f, "TwoPair       {}, {}", r1, r2),
            Strength::ThreeOAK(r) => write!(f, "ThreeOfAKind  {}", r),
            Strength::Straight(r) => write!(f, "Straight      {}", r),
            Strength::Flush(r) => write!(f, "Flush         {}", r),
            Strength::FullHouse(r1, r2) => write!(f, "FullHouse     {}, {}", r1, r2),
            Strength::FourOAK(r) => write!(f, "FourOfAKind   {}", r),
            Strength::StraightFlush(r) => write!(f, "StraightFlush {}", r),
        }
    }
}

impl From<&Strength> for u8 {
    fn from(strength: &Strength) -> u8 {
        match strength {
            Strength::MUCK => u8::MIN,
            Strength::MAX => u8::MAX,
            Strength::HighCard(_) => 1,
            Strength::OnePair(_) => 2,
            Strength::TwoPair(_, _) => 3,
            Strength::ThreeOAK(_) => 4,
            Strength::Straight(_) => 5,
            Strength::Flush(_) => 6,
            Strength::FullHouse(_, _) => 7,
            Strength::FourOAK(_) => 8,
            Strength::StraightFlush(_) => 9,
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
