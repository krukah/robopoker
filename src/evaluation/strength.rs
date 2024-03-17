#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Strength {
    HighCard(Rank),
    OnePair(Rank),
    TwoPair(Rank, Rank),
    ThreeOfAKind(Rank),
    Straight(Rank),
    Flush(Rank),
    FullHouse(Rank, Rank),
    FourOfAKind(Rank),
    StraightFlush(Rank),
    INFINITE,
}

impl Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strength::HighCard(r) => write!(f, "HighCard      {}", r),
            Strength::OnePair(r) => write!(f, "OnePair       {}", r),
            Strength::TwoPair(r1, r2) => write!(f, "TwoPair       {}, {}", r1, r2),
            Strength::ThreeOfAKind(r) => write!(f, "ThreeOfAKind  {}", r),
            Strength::Straight(r) => write!(f, "Straight      {}", r),
            Strength::Flush(r) => write!(f, "Flush         {}", r),
            Strength::FullHouse(r1, r2) => write!(f, "FullHouse     {}, {}", r1, r2),
            Strength::FourOfAKind(r) => write!(f, "FourOfAKind   {}", r),
            Strength::StraightFlush(r) => write!(f, "StraightFlush {}", r),
            Strength::INFINITE => unreachable!(),
        }
    }
}

impl Ord for Strength {
    fn cmp(&self, other: &Self) -> Ordering {
        match usize::from(self).cmp(&usize::from(other)) {
            Ordering::Equal => {
                //  compare the contained Ranks
                match (self, other) {
                    // compare primary ranks
                    (Strength::StraightFlush(a), Strength::StraightFlush(b))
                    | (Strength::FourOfAKind(a), Strength::FourOfAKind(b))
                    | (Strength::Flush(a), Strength::Flush(b))
                    | (Strength::Straight(a), Strength::Straight(b))
                    | (Strength::ThreeOfAKind(a), Strength::ThreeOfAKind(b))
                    | (Strength::OnePair(a), Strength::OnePair(b))
                    | (Strength::HighCard(a), Strength::HighCard(b)) => a.cmp(b),
                    // compare secondary pairs
                    (Strength::TwoPair(a1, a2), Strength::TwoPair(b1, b2))
                    | (Strength::FullHouse(a1, a2), Strength::FullHouse(b1, b2)) => {
                        match a1.cmp(b1) {
                            Ordering::Equal => a2.cmp(b2),
                            primary => primary,
                        }
                    }
                    _ => unreachable!(),
                }
            }
            strength => strength,
        }
    }
}

impl From<&Strength> for usize {
    fn from(hand_rank: &Strength) -> usize {
        match hand_rank {
            Strength::HighCard(_) => 0,
            Strength::OnePair(_) => 1,
            Strength::TwoPair(_, _) => 2,
            Strength::ThreeOfAKind(_) => 3,
            Strength::Straight(_) => 4,
            Strength::Flush(_) => 5,
            Strength::FullHouse(_, _) => 6,
            Strength::FourOfAKind(_) => 7,
            Strength::StraightFlush(_) => 8,
            Strength::INFINITE => 9,
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
