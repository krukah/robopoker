#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
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

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Kickers(pub Vec<Rank>);

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct FullStrength(pub Strength, pub Kickers);

impl Strength {
    pub fn rank(&self) -> Rank {
        match self {
            Strength::StraightFlush(r)
            | Strength::FullHouse(r, _)
            | Strength::TwoPair(r, _)
            | Strength::Straight(r)
            | Strength::ThreeOAK(r)
            | Strength::HighCard(r)
            | Strength::OnePair(r)
            | Strength::FourOAK(r)
            | Strength::Flush(r) => *r,
            Strength::MUCK | Strength::MAX => unreachable!(),
        }
    }
    pub fn secondary(&self) -> Rank {
        match self {
            Strength::TwoPair(_, r) | Strength::FullHouse(_, r) => *r,
            x => x.rank(),
        }
    }
}

impl Ord for Strength {
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::Equal
            .then_with(|| u8::from(self).cmp(&u8::from(other)))
            .then_with(|| self.rank().cmp(&other.rank()))
            .then_with(|| self.secondary().cmp(&other.secondary()))
            .then(Ordering::Equal)
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

impl Ord for FullStrength {
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::Equal
            .then_with(|| self.0.cmp(&other.0))
            .then_with(|| self.1.cmp(&other.1))
    }
}

impl Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strength::MUCK => write!(f, ""),
            Strength::MAX => unreachable!(),
            Strength::FullHouse(r1, r2) => write!(f, "FullHouse     {}, {}", r1, r2),
            Strength::TwoPair(r1, r2) => write!(f, "TwoPair       {}, {}", r1, r2),
            Strength::HighCard(r) => write!(f, "HighCard      {}", r),
            Strength::OnePair(r) => write!(f, "OnePair       {}", r),
            Strength::ThreeOAK(r) => write!(f, "ThreeOfAKind  {}", r),
            Strength::Straight(r) => write!(f, "Straight      {}", r),
            Strength::FourOAK(r) => write!(f, "FourOfAKind   {}", r),
            Strength::Flush(r) => write!(f, "Flush         {}", r),
            Strength::StraightFlush(r) => write!(f, "StraightFlush {}", r),
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

impl Display for FullStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<18}", self.0)
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

use crate::cards::rank::Rank;
use std::cmp::Ordering;
use std::fmt::Display;
