use super::card::Card;
use super::hand::Hand;
use super::hole::Hole;
use super::street::Street;

/// A mutable deck of cards supporting random draws.
///
/// Wraps a [`Hand`] representing the remaining cards, with methods for
/// randomly drawing cards and dealing hands. Used for Monte Carlo sampling
/// and game simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deck(Hand);

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    /// Creates a fresh 52-card deck (or 36 for short deck).
    pub fn new() -> Self {
        Self(Hand::from(Hand::mask()))
    }
    /// Tests whether a card is still in the deck.
    pub fn contains(&self, card: &Card) -> bool {
        self.0.contains(card)
    }
    /// Draws and removes a uniformly random card from the deck.
    ///
    /// Unlike `Hand::next()` which is deterministic, this samples
    /// uniformly for Monte Carlo simulation.
    pub fn draw(&mut self) -> Card {
        debug_assert!(self.0.size() > 0);
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
    /// Deals the appropriate number of cards for the next street.
    pub fn deal(&mut self, street: Street) -> Hand {
        (0..street.next().n_revealed())
            .map(|_| self.draw())
            .map(Hand::from)
            .fold(Hand::empty(), Hand::add)
    }
    /// Deals two cards as a player's hole cards.
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

impl Iterator for Deck {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.draw())
    }
}
