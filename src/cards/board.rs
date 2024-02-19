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
    pub fn new() -> Board {
        Board {
            cards: Vec::with_capacity(5),
            street: Street::Pre,
        }
    }
}
