use super::card::Card;
use super::hand::Hand;
use super::hole::Hole;
use rand::Rng;

/// Deck extends much of Hand functionality, with ability to remove cards from itself. Random selection via ::draw(), or sequential via ::flip().
#[derive(Debug, Clone, Copy)]
pub struct Deck(Hand);

impl From<Deck> for Hand {
    fn from(deck: Deck) -> Self {
        deck.0
    }
}
impl From<Hand> for Deck {
    fn from(hand: Hand) -> Self {
        Self(hand)
    }
}

impl Deck {
    pub fn new() -> Self {
        Self(Hand::from((1 << 52) - 1))
    }

    /// remove a specific card from the deck
    pub fn remove(&mut self, card: Card) {
        let this = u64::from(self.0);
        let card = u64::from(card);
        let mask = !(1 << card);
        self.0 = Hand::from(this & mask);
    }

    /// remove a random card from the deck
    pub fn draw(&mut self) -> Card {
        assert!(self.0.size() > 0);
        let n = self.0.size();
        let i = rand::thread_rng().gen_range(0..n);
        let mut deck = u64::from(self.0);
        let mut ones = 0;
        while ones < i {
            deck &= deck - 1;
            ones += 1;
        }
        let card = deck.trailing_zeros() as u8;
        let card = Card::from(card);
        self.remove(card);
        card
    }

    /// remove two cards from the deck
    /// to deal as a Hole
    pub fn hole(&mut self) -> Hole {
        let a = self.draw();
        let b = self.draw();
        Hole::from((a, b))
    }
}
