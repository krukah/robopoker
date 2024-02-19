use super::card::Card;

pub enum Street {
    Pre,
    Flop,
    Turn,
    River,
    Showdown,
}

pub struct Board {
    pub cards: Vec<Card>, // presize
    pub street: Street,
}

impl Board {
    pub fn accept(&mut self, card: Card) {
        self.cards.push(card);
    }
}
