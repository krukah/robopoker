use super::card::Card;
use super::hand::Hand;
use super::observation::Observation;

/// A player's two private hole cards.
///
/// Wraps a [`Hand`] with the constraint that exactly two cards are present.
/// Hole cards determine a player's starting equity and strategic options.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Hole(Hand);

impl std::fmt::Display for Hole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Hand> for Hole {
    fn from(hand: Hand) -> Self {
        debug_assert!(hand.size() == 2);
        Self(hand)
    }
}
impl From<Hole> for Hand {
    fn from(hole: Hole) -> Self {
        hole.0
    }
}
impl IntoIterator for Hole {
    type Item = Card;
    type IntoIter = <Hand as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Observation> for Hole {
    fn from(obs: Observation) -> Self {
        Self(*obs.pocket())
    }
}

impl From<(Card, Card)> for Hole {
    fn from(cards: (Card, Card)) -> Self {
        let a = u64::from(cards.0);
        let b = u64::from(cards.1);
        debug_assert!(a != b);
        Self(Hand::from(a | b))
    }
}

impl TryFrom<&str> for Hole {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let hand = Hand::try_from(s)?;
        match hand.size() {
            2 => Ok(Self(hand)),
            _ => Err("hand must contain exactly two cards".into()),
        }
    }
}

impl serde::Serialize for Hole {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for Hole {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(d)?;
        Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}
