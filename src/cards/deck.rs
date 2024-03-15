#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>, // presize
}

impl Deck {
    pub fn new() -> Self {
        let mut this = Self {
            cards: (0u8..52).map(|n| Card::from(n)).collect(),
        };
        this.shuffle();
        this
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }
}

// u64 isomorphism
impl From<u64> for Deck {
    fn from(n: u64) -> Self {
        Self {
            cards: (0u8..52)
                .filter(|i| n & (1 << i) != 0)
                .map(|i| Card::from(i))
                .collect(),
        }
    }
}
impl From<Deck> for u64 {
    fn from(deck: Deck) -> u64 {
        deck.cards.iter().map(|c| u64::from(*c)).sum()
    }
}

use super::card::Card;
use rand::seq::SliceRandom;
use rand::thread_rng;
