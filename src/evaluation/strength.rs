#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
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
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Strength {
    hand: Value,
    kickers: Kickers,
}
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Kickers(pub Vec<Rank>);

impl Strength {
    pub fn new(hand: Value, kickers: Kickers) -> Self {
        Strength { hand, kickers }
    }
    pub fn kickers(&self) -> &Kickers {
        &self.kickers
    }
}

impl Value {
    pub fn primary(&self) -> Rank {
        match self {
            Value::StraightFlush(r, ..)
            | Value::FullHouse(r, ..)
            | Value::TwoPair(r, ..)
            | Value::Straight(r, ..)
            | Value::ThreeOAK(r, ..)
            | Value::HighCard(r, ..)
            | Value::OnePair(r, ..)
            | Value::FourOAK(r, ..)
            | Value::Flush(r, ..) => *r,
            Value::MAX => unreachable!(),
        }
    }
    pub fn secondary(&self) -> Rank {
        match self {
            Value::TwoPair(_, r) | Value::FullHouse(_, r) => *r,
            x => x.primary(),
        }
    }
}

impl Ord for Value {
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
            .then_with(|| self.kickers().cmp(&other.kickers))
    }
}

impl From<Hand> for Strength {
    fn from(hand: Hand) -> Self {
        todo!("migrate LazyEvaluator implementation into infallible Hand -> Strength map")
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::MAX => unreachable!(),
            Value::FullHouse(r1, r2) => write!(f, "FullHouse     {}, {}", r1, r2),
            Value::TwoPair(r1, r2) => write!(f, "TwoPair       {}, {}", r1, r2),
            Value::HighCard(r) => write!(f, "HighCard      {}", r),
            Value::OnePair(r) => write!(f, "OnePair       {}", r),
            Value::ThreeOAK(r) => write!(f, "ThreeOfAKind  {}", r),
            Value::Straight(r) => write!(f, "Straight      {}", r),
            Value::FourOAK(r) => write!(f, "FourOfAKind   {}", r),
            Value::Flush(r) => write!(f, "Flush         {}", r),
            Value::StraightFlush(r) => write!(f, "StraightFlush {}", r),
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

impl From<&Value> for u8 {
    fn from(strength: &Value) -> u8 {
        match strength {
            Value::MAX => u8::MAX,
            Value::HighCard(_) => 1,
            Value::OnePair(_) => 2,
            Value::TwoPair(_, _) => 3,
            Value::ThreeOAK(_) => 4,
            Value::Straight(_) => 5,
            Value::Flush(_) => 6,
            Value::FullHouse(_, _) => 7,
            Value::FourOAK(_) => 8,
            Value::StraightFlush(_) => 9,
        }
    }
}

use crate::cards::hand::Hand;
use crate::cards::rank::Rank;
use std::cmp::Ordering;
use std::fmt::Display;
