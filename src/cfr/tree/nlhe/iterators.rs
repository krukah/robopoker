use crate::cards::card::Card;

type Hand = u64;

/// A memory-efficient deterministic Card Iterator.
#[derive(Default)]
pub struct NoRemove {
    card: Option<Card>,
}
impl Iterator for NoRemove {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        self.card = match self.card {
            None => Some(Card::MIN),
            Some(Card::MAX) => None,
            Some(card) => Some(Card::from(u8::from(card) + 1)),
        };
        self.card
    }
}
impl From<Card> for NoRemove {
    fn from(card: Card) -> Self {
        Self { card: Some(card) }
    }
}

/// A memory-efficient deterministic Card Iterator. Can remove & insert cards.
#[derive(Default)]
pub struct DoRemove {
    mask: Hand,
    deck: NoRemove,
}
impl DoRemove {
    pub fn remove(&mut self, card: Card) {
        self.mask |= Hand::from(card);
    }
    pub fn insert(&mut self, card: Card) {
        self.mask &= !Hand::from(card);
    }
}
impl Iterator for DoRemove {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        self.deck
            .by_ref()
            .find(|draw| Hand::from(*draw) & (self.mask) == 0)
    }
}

/// A memory-efficient determistic Hand Iterator.
pub struct HandIterator {
    mask: Hand,
    hand: Hand,
    deck: DoRemove,
}
impl Iterator for HandIterator {
    type Item = Hand;
    fn next(&mut self) -> Option<Self::Item> {
        self.hand = match self.hand {
            0 => 1,
            0x1_0000_0000_0000_0000 => 0,
            hand => hand << 1,
        };
        self.deck.mask = self.hand;
        self.deck
            .by_ref()
            .find(|draw| Hand::from(*draw) & (self.mask) == 0)
            .into()
    }
}
