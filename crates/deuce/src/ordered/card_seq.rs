use crate::*;

/// Zero-allocation iterator yielding cards in permuted order.
///
/// Produced by [`Lehmer::arrange`] or [`HandSeq::cards`]. Holds a stack-allocated snapshot of
/// the canonical cards and the decoded index permutation.
///
/// [`HandSeq::cards`]: crate::HandSeq::cards
/// [`Lehmer::arrange`]: crate::Lehmer::arrange
#[derive(Debug, Clone, Copy)]
pub struct CardSeq {
    cards: [Card; 5],
    order: [u8; 5],
    index: u8,
    count: u8,
}

impl CardSeq {
    pub(crate) fn new(cards: [Card; 5], order: [u8; 5], count: u8) -> Self {
        Self {
            cards,
            order,
            index: 0,
            count,
        }
    }

    fn cards(&self) -> &[Card; 5] {
        &self.cards
    }

    fn order(&self) -> &[u8; 5] {
        &self.order
    }

    fn index(&self) -> u8 {
        self.index
    }

    fn index_mut(&mut self) -> &mut u8 {
        &mut self.index
    }

    fn end(&self) -> u8 {
        self.count
    }

    fn end_mut(&mut self) -> &mut u8 {
        &mut self.count
    }

    fn card(&self, i: u8) -> Card {
        self.cards()[self.order()[i as usize] as usize]
    }
}

impl Iterator for CardSeq {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        (self.index() < self.end()).then(|| {
            let card = self.card(self.index());
            *self.index_mut() += 1;
            card
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (self.end() - self.index()) as usize;
        (n, Some(n))
    }
}
impl DoubleEndedIterator for CardSeq {
    fn next_back(&mut self) -> Option<Card> {
        (self.index() < self.end()).then(|| {
            *self.end_mut() -= 1;
            self.card(self.end())
        })
    }
}
impl ExactSizeIterator for CardSeq {}
impl std::iter::FusedIterator for CardSeq {}

impl FromIterator<CardSeq> for Hand {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = CardSeq>,
    {
        iter.into_iter().flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_size() {
        let hand = Hand::try_from("2c 5d Th Qs As").unwrap();
        let mut seq = Lehmer::from(42).arrange(hand);
        assert_eq!(seq.len(), 5);
        seq.next();
        assert_eq!(seq.len(), 4);
        seq.next_back();
        assert_eq!(seq.len(), 3);
    }
    #[test]
    fn double_ended() {
        let hand = Hand::try_from("2c 5d Th Qs As").unwrap();
        let perm = Lehmer::from(77);
        let forward = perm.arrange(hand).collect::<Vec<_>>();
        let backward = perm.arrange(hand).rev().collect::<Vec<_>>();
        assert_eq!(forward.into_iter().rev().collect::<Vec<_>>(), backward);
    }
    #[test]
    fn fused() {
        let hand = Hand::try_from("7h Kd").unwrap();
        let mut seq = Lehmer::from(1).arrange(hand);
        seq.next();
        seq.next();
        assert_eq!(seq.next(), None);
        assert_eq!(seq.next(), None);
        assert_eq!(seq.next(), None);
    }
    #[test]
    fn collect_into_hand() {
        let hand = Hand::try_from("2c 5d Th Qs As").unwrap();
        let seq = Lehmer::from(42).arrange(hand);
        assert_eq!(seq.collect::<Hand>(), hand);
    }
    #[test]
    fn empty_seq() {
        let seq = Lehmer::identity().arrange(Hand::empty());
        assert_eq!(seq.len(), 0);
        assert_eq!(seq.collect::<Vec<_>>(), vec![]);
    }
}
