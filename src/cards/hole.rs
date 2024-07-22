use super::card::Card;

#[derive(Debug, Clone)]
pub struct Hole {
    pub cards: Vec<Card>, // presize
}

impl Hole {
    pub fn new() -> Hole {
        Hole {
            cards: Vec::with_capacity(2),
        }
    }
}

impl std::fmt::Display for Hole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.cards[0], self.cards[1])
    }
}
