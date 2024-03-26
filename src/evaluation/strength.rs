#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum BestHand {
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
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Strength {
    hand: BestHand,
    kickers: Kickers,
}
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Kickers(pub Vec<Rank>);

impl Strength {
    pub fn new(hand: BestHand, kickers: Kickers) -> Self {
        Strength { hand, kickers }
    }
    pub fn rank(&self) -> Rank {
        match self.hand {
            BestHand::StraightFlush(r, ..)
            | BestHand::FullHouse(r, ..)
            | BestHand::TwoPair(r, ..)
            | BestHand::Straight(r, ..)
            | BestHand::ThreeOAK(r, ..)
            | BestHand::HighCard(r, ..)
            | BestHand::OnePair(r, ..)
            | BestHand::FourOAK(r, ..)
            | BestHand::Flush(r, ..) => r,
            BestHand::MAX => unreachable!(),
        }
    }
    pub fn secondary(&self) -> Rank {
        match self.hand {
            BestHand::TwoPair(_, r) | BestHand::FullHouse(_, r) => r,
            _ => self.rank(),
        }
    }
    pub fn kickers(&self) -> Kickers {
        self.kickers.clone()
    }
}

impl BestHand {
    pub fn primary(&self) -> Rank {
        match self {
            BestHand::StraightFlush(r, ..)
            | BestHand::FullHouse(r, ..)
            | BestHand::TwoPair(r, ..)
            | BestHand::Straight(r, ..)
            | BestHand::ThreeOAK(r, ..)
            | BestHand::HighCard(r, ..)
            | BestHand::OnePair(r, ..)
            | BestHand::FourOAK(r, ..)
            | BestHand::Flush(r, ..) => *r,
            BestHand::MAX => unreachable!(),
        }
    }
    pub fn secondary(&self) -> Rank {
        match self {
            BestHand::TwoPair(_, r) | BestHand::FullHouse(_, r) => *r,
            x => x.primary(),
        }
    }
}

impl Ord for BestHand {
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::Equal
            .then_with(|| u8::from(self).cmp(&u8::from(other)))
            .then_with(|| self.primary().cmp(&other.primary()))
            .then_with(|| self.secondary().cmp(&other.secondary()))
    }
}
impl Ord for Kickers {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| a.cmp(b))
            .find(|&x| x != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    }
}

impl Ord for Strength {
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::Equal
            .then_with(|| self.hand.cmp(&&other.hand))
            .then_with(|| self.kickers.cmp(&other.kickers))
    }
}

impl Display for BestHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BestHand::MAX => unreachable!(),
            BestHand::FullHouse(r1, r2) => write!(f, "FullHouse     {}, {}", r1, r2),
            BestHand::TwoPair(r1, r2) => write!(f, "TwoPair       {}, {}", r1, r2),
            BestHand::HighCard(r) => write!(f, "HighCard      {}", r),
            BestHand::OnePair(r) => write!(f, "OnePair       {}", r),
            BestHand::ThreeOAK(r) => write!(f, "ThreeOfAKind  {}", r),
            BestHand::Straight(r) => write!(f, "Straight      {}", r),
            BestHand::FourOAK(r) => write!(f, "FourOfAKind   {}", r),
            BestHand::Flush(r) => write!(f, "Flush         {}", r),
            BestHand::StraightFlush(r) => write!(f, "StraightFlush {}", r),
        }
    }
}

impl Display for Kickers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in &self.0 {
            write!(f, "{} ", rank)?;
        }
        Ok(())
    }
}

impl Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<18}", self.hand)
    }
}

impl From<&BestHand> for u8 {
    fn from(strength: &BestHand) -> u8 {
        match strength {
            BestHand::MAX => u8::MAX,
            BestHand::HighCard(_) => 1,
            BestHand::OnePair(_) => 2,
            BestHand::TwoPair(_, _) => 3,
            BestHand::ThreeOAK(_) => 4,
            BestHand::Straight(_) => 5,
            BestHand::Flush(_) => 6,
            BestHand::FullHouse(_, _) => 7,
            BestHand::FourOAK(_) => 8,
            BestHand::StraightFlush(_) => 9,
        }
    }
}

use crate::cards::rank::Rank;
use std::cmp::Ordering;
use std::fmt::Display;
