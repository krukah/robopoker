use super::card::Card;
use super::hand::Hand;
use super::observation::Observation;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Hole(Hand);

impl Hole {
    pub fn empty() -> Self {
        Self(Hand::empty())
    }
}

impl std::fmt::Display for Hole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Hand> for Hole {
    fn from(hand: Hand) -> Self {
        assert!(hand.size() == 2);
        Self(hand)
    }
}
impl From<Hole> for Hand {
    fn from(hole: Hole) -> Self {
        hole.0
    }
}

impl From<Observation> for Hole {
    fn from(obs: Observation) -> Self {
        Self(Hand::from(obs.pocket().clone()))
    }
}

impl From<(Card, Card)> for Hole {
    fn from(cards: (Card, Card)) -> Self {
        let a = u64::from(cards.0);
        let b = u64::from(cards.1);
        assert!(a != b);
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
