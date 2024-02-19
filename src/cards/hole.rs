use super::card::Card;

#[derive(Debug, Clone)]
pub struct Hole {
    cards: Vec<Card>, // presize
}

impl Hole {
    pub fn new() -> Hole {
        Hole {
            cards: Vec::with_capacity(2),
        }
    }
}
