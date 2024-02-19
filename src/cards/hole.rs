use super::card::Card;

pub struct Hole {
    cards: Vec<Card>, // presize
}

impl Hole {
    pub fn new() -> Hole {
        Hole {
            cards: Vec::with_capacity(2),
        }
    }

    pub fn accept(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn equity(holes: Vec<Hole>) -> f32 {
        todo!()
    }
}
