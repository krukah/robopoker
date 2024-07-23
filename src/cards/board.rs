use super::card::Card;
use super::street::Street;

///
#[derive(Debug, Clone)]
pub struct Board {
    pub cards: Vec<Card>, // presize
    pub street: Street,
}

impl Board {
    pub fn new() -> Board {
        Board {
            cards: Vec::with_capacity(5),
            street: Street::Pref,
        }
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for card in self.cards.iter() {
            write!(f, "{}  ", card)?;
        }
        Ok(())
    }
}
