use super::card::Card;
use super::hand::Hand;
use super::hole::Hole;
use super::street::Street;

/// Deck extends much of Hand functionality, with ability to remove cards from itself. Random selection via ::draw(), or sequential via ::flip().
#[derive(Debug, Clone, Copy)]
pub struct Deck(Hand);
impl Deck {
    pub fn new() -> Self {
        Self(Hand::from(Hand::mask()))
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.0.contains(card)
    }

    // #[cfg(feature = "entropy")]
    /// remove a random card from the deck.
    /// different from Hand::draw() since that removes
    /// highest card deterministically
    pub fn draw(&mut self) -> Card {
        let n = self.0.size();
        let i = rand::random_range(0..n) as u8;
        let mut ones = 0u8;
        let mut deck = u64::from(self.0);
        let mut card = u64::from(self.0).trailing_zeros() as u8;
        while ones < i {
            card = deck.trailing_zeros() as u8;
            deck = deck & (deck - 1);
            ones = ones + 1;
        }
        let card = Card::from(card);
        self.0.remove(card);
        card
    }

    /// only needed for Flop, but the creation of a Hand is well-generalized
    pub fn deal(&mut self, street: Street) -> Hand {
        (0..street.n_revealed())
            .map(|_| self.draw())
            .fold(Hand::empty(), |h, c| Hand::add(h, Hand::from(c)))
    }

    /// remove two cards from the deck
    /// to deal as a Hole
    pub fn hole(&mut self) -> Hole {
        let a = self.draw();
        let b = self.draw();
        Hole::from((a, b))
    }
}

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
