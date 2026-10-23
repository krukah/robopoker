use crate::*;

/// Poker hand category from high card to straight flush.
///
/// Each variant carries the primary rank(s) that define the hand. Kickers are tracked
/// separately in [`Kickers`].
///
/// [`Kickers`]: crate::Kickers
#[cfg(feature = "shortdeck")]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum Ranking {
    HighCard(Rank),
    OnePair(Rank),
    TwoPair(Rank, Rank),
    ThreeOAK(Rank),
    Straight(Rank),
    FullHouse(Rank, Rank),
    Flush(Rank),
    FourOAK(Rank),
    StraightFlush(Rank),
    MAX,
}
/// Poker hand category from high card to straight flush.
///
/// Each variant carries the primary rank(s) that define the hand. Kickers are tracked
/// separately in [`Kickers`].
///
/// [`Kickers`]: crate::Kickers
#[cfg(not(feature = "shortdeck"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum Ranking {
    HighCard(Rank),
    OnePair(Rank),
    TwoPair(Rank, Rank),
    ThreeOAK(Rank),
    Straight(Rank),
    Flush(Rank),
    FullHouse(Rank, Rank),
    FourOAK(Rank),
    StraightFlush(Rank),
    MAX,
}

impl Ranking {
    /// Number of deuce cards used for this hand type.
    ///
    /// A flush needs four: it is compared over all five of its cards, and only
    /// its highest is carried in the variant. A straight and a straight flush
    /// need none — their top card determines the other four.
    pub fn n_kickers(&self) -> usize {
        match self {
            Ranking::HighCard(_) | Ranking::Flush(_) => 4,
            Ranking::OnePair(_) => 3,
            Ranking::ThreeOAK(_) => 2,
            Ranking::FourOAK(_) | Ranking::TwoPair(_, _) => 1,
            _ => 0,
        }
    }

    /// Bitmask of ranks NOT used in the hand's primary ranking.
    ///
    /// Used to extract deuce cards from the remaining ranks. A flush masks off
    /// only its high card; which *suit* the rest may come from is the caller's
    /// job — see `Evaluator::eligible`.
    pub fn mask(&self) -> u16 {
        match *self {
            Ranking::TwoPair(hi, lo) => !(u16::from(hi) | u16::from(lo)),
            Ranking::HighCard(hi)
            | Ranking::OnePair(hi)
            | Ranking::FourOAK(hi)
            | Ranking::ThreeOAK(hi)
            | Ranking::Flush(hi) => !(u16::from(hi)),
            _ => unreachable!(),
        }
    }
}

impl Ranking {
    /// The tile-sized abbreviation — at most three characters, for surfaces
    /// that get one cell and no more (the tree's settlement column). Pairs
    /// with [`Self::label`], which is the same fact with room to breathe.
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::HighCard(_) => "HI",
            Self::OnePair(_) => "1P",
            Self::TwoPair(_, _) => "2P",
            Self::ThreeOAK(_) => "3OK",
            Self::Straight(_) => "STR",
            Self::Flush(_) => "FLS",
            Self::FullHouse(_, _) => "FUL",
            Self::FourOAK(_) => "4OK",
            Self::StraightFlush(_) => "STF",
            Self::MAX => unreachable!(),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::HighCard(_) => "High Card",
            Self::OnePair(_) => "Pair",
            Self::TwoPair(_, _) => "Two Pair",
            Self::ThreeOAK(_) => "Trips",
            Self::Straight(_) => "Straight",
            Self::Flush(_) => "Flush",
            Self::FullHouse(_, _) => "Full House",
            Self::FourOAK(_) => "Quads",
            Self::StraightFlush(_) => "Str. Flush",
            Self::MAX => unreachable!(),
        }
    }
}
impl std::fmt::Display for Ranking {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Ranking::MAX => unreachable!(),
            Ranking::FullHouse(r1, r2) => write!(f, "FullHouse     {r1}{r2}"),
            Ranking::TwoPair(r1, r2) => write!(f, "TwoPair       {r1}{r2}"),
            Ranking::HighCard(r) => write!(f, "HighCard      {r} "),
            Ranking::OnePair(r) => write!(f, "OnePair       {r} "),
            Ranking::ThreeOAK(r) => write!(f, "ThreeOfAKind  {r} "),
            Ranking::Straight(r) => write!(f, "Straight      {r} "),
            Ranking::FourOAK(r) => write!(f, "FourOfAKind   {r} "),
            Ranking::Flush(r) => write!(f, "Flush         {r} "),
            Ranking::StraightFlush(r) => write!(f, "StraightFlush {r} "),
        }
    }
}
