use super::card::Card;

#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>, // presize
}

impl Deck {
    pub fn new() -> Deck {
        Deck {
            cards: (0..52).map(Card::from).collect(),
        }
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn shuffle(&mut self) {
        todo!()
    }
}
