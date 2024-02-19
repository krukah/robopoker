use super::card::Card;

#[derive(Debug, Clone, Copy)]
pub enum Street {
    Pre,
    Flop,
    Turn,
    River,
    Showdown,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub cards: Vec<Card>, // presize
    pub street: Street,
}

impl Board {
    pub fn accept(&mut self, card: Card) {
        self.cards.push(card);
    }
}
