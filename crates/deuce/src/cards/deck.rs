use crate::*;

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
    /// Unlike `Hand::next()` which is deterministic, this samples uniformly for Monte Carlo
    /// simulation.
    ///
    /// The `rng` is explicit on purpose.
    pub fn draw<R>(&mut self, rng: &mut R) -> Card
    where
        R: rand::Rng + ?Sized,
    {
        debug_assert!(self.0.size() > 0);
        let card = Card::from(
            (0..rng.random_range(0..self.0.size()))
                .fold(u64::from(self.0), |deck, _| deck & (deck - 1))
                .trailing_zeros() as u8,
        );
        self.0.remove(card);
        card
    }
    /// Deals the appropriate number of cards for the next street.
    pub fn deal<R>(&mut self, street: Street, rng: &mut R) -> Hand
    where
        R: rand::Rng + ?Sized,
    {
        (0..street.next().n_revealed())
            .map(|_| self.draw(rng))
            .map(Hand::from)
            .fold(Hand::empty(), Hand::add)
    }
    /// Deals two cards as a player's hole cards.
    pub fn hole<R>(&mut self, rng: &mut R) -> Hole
    where
        R: rand::Rng + ?Sized,
    {
        let a = self.draw(rng);
        let b = self.draw(rng);
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
        Some(self.draw(&mut rand::rng()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use std::collections::HashMap;
    use std::collections::HashSet;

    /// Every card must be reachable from a fresh deck, and reachable equally often.
    #[test]
    fn draw_is_uniform_over_a_full_deck() {
        const TRIALS: usize = 52 * 4096;
        let ref mut rng = rand::rngs::StdRng::seed_from_u64(0xDEC4);
        let mut hist = [0usize; 52];
        for _ in 0..TRIALS {
            hist[usize::from(u8::from(Deck::new().draw(rng)))] += 1;
        }
        for (i, &count) in hist.iter().enumerate() {
            let ratio = count as f64 / (TRIALS as f64 / 52.0);
            assert!(0.85 < ratio && ratio < 1.15, "{} drawn {count} times (ratio {ratio:.3})", Card::from(i as u8));
        }
    }

    /// Draining a deck must yield each card exactly once — no repeat from a
    /// double-counted low bit, no card stranded at the top.
    #[test]
    fn draining_yields_every_card_once() {
        let ref mut rng = rand::rngs::StdRng::seed_from_u64(0xDEC5);
        let mut deck = Deck::new();
        assert_eq!(52, (0..52).map(|_| deck.draw(rng)).collect::<HashSet<_>>().len());
        assert_eq!(0, Hand::from(deck).size());
    }

    /// A depleted deck is still uniform over what remains, including its own
    /// highest card.
    #[test]
    fn draw_is_uniform_over_a_depleted_deck() {
        let ref mut rng = rand::rngs::StdRng::seed_from_u64(0xDEC6);
        let hand = Hand::from(Card::parse("2c 7d Th As").unwrap());
        let mut hist = HashMap::new();
        for _ in 0..4096 {
            *hist.entry(Deck::from(hand).draw(rng)).or_insert(0usize) += 1;
        }
        assert_eq!(4, hist.len());
        for (card, count) in hist {
            assert!(768 < count && count < 1280, "{card} drawn {count} times");
        }
    }
}
