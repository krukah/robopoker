use super::card::Card;

pub struct Deck {
    cards: Vec<Card>, // presize
}

impl Deck {
    pub fn new() -> Deck {
        Deck {
            cards: (0..52).map(Card::from).collect(),
        }
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn shuffle(&mut self) {
        todo!()
    }
}
