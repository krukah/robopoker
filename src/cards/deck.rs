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

    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }
}
use super::card::Card;
use rand::seq::SliceRandom;
use rand::thread_rng;
