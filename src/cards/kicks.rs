use super::rank::Rank;

/// A hand's kicker cards.
///
/// This is a simplified version of the hand's value, and does not include the hand's kicker cards.
/// The value is ordered by the hand's strength, and the kicker cards are used to break ties.
/// WARNING: Implementation of Ord will not correctly compare Suits.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Kickers(u32);

/// u32 isomorphism
/// importantly, we ignore (not erase) the Suit bits
impl From<Kickers> for u32 {
    fn from(k: Kickers) -> Self {
        k.0
    }
}
impl From<u32> for Kickers {
    fn from(n: u32) -> Self {
        Self(n)
    }
}

/// Vec<Rank> isomorphism
///
/// [2c, Ts, Jc, Js, Jd, Jh]
/// xxxxxxxxxxxx 000001100000001
impl From<Kickers> for Vec<Rank> {
    fn from(k: Kickers) -> Self {
        let mut value = k.0;
        let mut index = 0u8;
        let mut ranks = Vec::new();
        while value > 0 {
            if value & 1 == 1 {
                ranks.push(Rank::from(index));
            }
            value = value >> 1;
            index = index + 1;
        }
        ranks
    }
}
impl From<Vec<Rank>> for Kickers {
    fn from(ranks: Vec<Rank>) -> Self {
        Self(ranks.iter().map(|r| u32::from(*r)).fold(0u32, |a, b| a | b))
    }
}

impl std::fmt::Display for Kickers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for rank in Vec::<Rank>::from(*self) {
            write!(f, "{} ", rank)?;
        }
        Ok(())
    }
}
