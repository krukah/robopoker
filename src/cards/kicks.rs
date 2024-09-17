use super::rank::Rank;

/// A hand's kicker cards.
///
/// This is a simplified version of the hand's value, and does not include the hand's kicker cards.
/// The value is ordered by the hand's strength, and the kicker cards are used to break ties.
/// WARNING: Implementation of Ord will not correctly compare Suits.
#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Kickers(u16);

/// u32 isomorphism
/// importantly, we ignore (not erase) the Suit bits
impl From<Kickers> for u16 {
    fn from(k: Kickers) -> Self {
        k.0
    }
}
impl From<u16> for Kickers {
    fn from(n: u16) -> Self {
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
        Self(
            ranks
                .into_iter()
                .map(|r| u16::from(r))
                .fold(0u16, |a, b| a | b),
        )
    }
}

impl std::fmt::Display for Kickers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for rank in Vec::<Rank>::from(self.clone()) {
            write!(f, "{} ", rank)?;
        }
        Ok(())
    }
}
