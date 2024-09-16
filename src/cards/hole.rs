use super::card::Card;
use super::hand::Hand;

#[derive(Debug, Clone, Copy)]
pub struct Hole(Hand);

impl Hole {
    pub fn new() -> Self {
        Self(Hand::from(3u64))
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

impl From<(Card, Card)> for Hole {
    fn from(cards: (Card, Card)) -> Self {
        let a = u64::from(cards.0);
        let b = u64::from(cards.1);
        let hand = Hand::from(a | b);
        Self(hand)
    }
}
