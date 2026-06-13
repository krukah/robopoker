use super::card::Card;
use super::card_seq::CardSeq;
use super::hand::Hand;
use super::perm::Perm;

/// An ordered hand: a [`Hand`] (the set) paired with a [`Perm`] (the deal order).
///
/// `Hand` is an unordered bitmask — it forgets insertion order. `HandSeq`
/// preserves that ordering so cards can be reconstructed in the sequence
/// they were dealt. Implements [`IntoIterator`] to yield a [`CardSeq`].
///
/// # Trait relationships
///
/// - [`IntoIterator`] → [`CardSeq`] (zero-allocation, ordered traversal)
/// - [`FromIterator<Card>`] — collect an ordered card sequence
/// - [`From<Hand>`] — upgrade with identity (canonical) ordering
/// - [`From<HandSeq>`] for [`Hand`] — downgrade, discarding order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandSeq {
    hand: Hand,
    perm: Perm,
}

impl HandSeq {
    /// Empty sequence (no cards).
    pub fn empty() -> Self {
        Self {
            hand: Hand::empty(),
            perm: Perm::identity(),
        }
    }
    /// The unordered card set.
    pub fn hand(&self) -> &Hand {
        &self.hand
    }
    /// The deal-order permutation.
    pub fn perm(&self) -> Perm {
        self.perm
    }
    /// Number of cards.
    pub fn size(&self) -> usize {
        self.hand.size()
    }
    /// Cards in deal order as a zero-allocation iterator.
    pub fn cards(&self) -> CardSeq {
        self.perm.arrange(self.hand)
    }
    /// Appends cards in canonical (ascending) order.
    pub fn add(&mut self, hand: Hand) {
        let mut buf = [Card::from(0u8); 5];
        let mut n = 0u8;
        for card in self.perm.arrange(self.hand) {
            buf[n as usize] = card;
            n += 1;
        }
        for card in hand {
            buf[n as usize] = card;
            n += 1;
        }
        self.hand = Hand::add(self.hand, hand);
        self.perm = Perm::of(&buf[..n as usize]);
    }
    /// Appends cards in the given deal order.
    pub fn deal(&mut self, cards: &[Card]) {
        let mut buf = [Card::from(0u8); 5];
        let mut n = 0u8;
        for card in self.perm.arrange(self.hand) {
            buf[n as usize] = card;
            n += 1;
        }
        for &card in cards {
            buf[n as usize] = card;
            n += 1;
        }
        self.hand = Hand::add(self.hand, cards.iter().copied().collect());
        self.perm = Perm::of(&buf[..n as usize]);
    }
    /// Resets to empty.
    pub fn clear(&mut self) {
        self.hand = Hand::empty();
        self.perm = Perm::identity();
    }
}

impl IntoIterator for HandSeq {
    type Item = Card;
    type IntoIter = CardSeq;

    fn into_iter(self) -> CardSeq {
        self.perm.arrange(self.hand)
    }
}
impl IntoIterator for &HandSeq {
    type Item = Card;
    type IntoIter = CardSeq;

    fn into_iter(self) -> CardSeq {
        self.perm.arrange(self.hand)
    }
}

impl FromIterator<Card> for HandSeq {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Card>,
    {
        let mut buf = [Card::from(0u8); 5];
        let mut n = 0u8;
        for card in iter {
            buf[n as usize] = card;
            n += 1;
        }
        debug_assert!(n <= 5);
        Self {
            hand: buf[..n as usize].iter().copied().collect(),
            perm: Perm::of(&buf[..n as usize]),
        }
    }
}

/// Upgrade with identity (canonical) ordering.
impl From<Hand> for HandSeq {
    fn from(hand: Hand) -> Self {
        Self {
            hand,
            perm: Perm::identity(),
        }
    }
}

/// Construct from an explicit `(Hand, Perm)` pair.
impl From<(Hand, Perm)> for HandSeq {
    fn from((hand, perm): (Hand, Perm)) -> Self {
        Self { hand, perm }
    }
}

/// Downgrade, discarding deal order.
impl From<HandSeq> for Hand {
    fn from(seq: HandSeq) -> Self {
        seq.hand
    }
}

impl std::fmt::Display for HandSeq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.cards().map(|c| format!("{c}")).collect::<Vec<String>>().join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let seq = HandSeq::empty();
        assert_eq!(seq.size(), 0);
        assert_eq!(seq.into_iter().count(), 0);
    }
    #[test]
    fn from_hand_is_canonical() {
        let hand = Hand::try_from("Qh 5d Td").unwrap();
        let seq = HandSeq::from(hand);
        assert_eq!(seq.into_iter().collect::<Vec<_>>(), Vec::<Card>::from(hand));
    }
    #[test]
    fn collect_preserves_order() {
        let cards = Card::parse("Qh 5d Td").unwrap();
        let seq = cards.iter().copied().collect::<HandSeq>();
        assert_eq!(seq.into_iter().collect::<Vec<_>>(), cards);
    }
    #[test]
    fn deal_preserves_order() {
        let mut seq = HandSeq::empty();
        let flop = Card::parse("Qh 5d Td").unwrap();
        seq.deal(&flop);
        assert_eq!(seq.into_iter().collect::<Vec<_>>(), flop);
    }
    #[test]
    fn deal_incremental() {
        let mut seq = HandSeq::empty();
        seq.deal(&Card::parse("Qh 5d Td").unwrap());
        seq.deal(&Card::parse("Ah").unwrap());
        seq.deal(&Card::parse("2c").unwrap());
        let expected = Card::parse("Qh 5d Td Ah 2c").unwrap();
        assert_eq!(seq.into_iter().collect::<Vec<_>>(), expected);
    }
    #[test]
    fn add_uses_canonical_order() {
        let mut seq = HandSeq::empty();
        seq.add(Hand::try_from("Qh 5d Td").unwrap());
        let canonical = Vec::<Card>::from(Hand::try_from("Qh 5d Td").unwrap());
        assert_eq!(seq.into_iter().collect::<Vec<_>>(), canonical);
    }
    #[test]
    fn into_hand_drops_order() {
        let cards = Card::parse("Qh 5d Td").unwrap();
        let seq = cards.iter().copied().collect::<HandSeq>();
        let hand = Hand::from(seq);
        assert_eq!(hand, Hand::try_from("Qh 5d Td").unwrap());
    }
    #[test]
    fn clear_resets() {
        let mut seq = Card::parse("Qh 5d Td").unwrap().into_iter().collect::<HandSeq>();
        seq.clear();
        assert_eq!(seq.size(), 0);
        assert_eq!(seq.perm(), Perm::identity());
    }
    #[test]
    fn ref_into_iter() {
        let seq = Card::parse("Td Qh 5d").unwrap().into_iter().collect::<HandSeq>();
        let first = (&seq).into_iter().collect::<Vec<_>>();
        let second = (&seq).into_iter().collect::<Vec<_>>();
        assert_eq!(first, second);
    }
}
