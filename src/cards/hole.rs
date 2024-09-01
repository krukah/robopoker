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
