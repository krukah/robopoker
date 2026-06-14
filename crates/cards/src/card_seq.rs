use super::card::Card;
use super::hand::Hand;

/// Zero-allocation iterator yielding cards in permuted order.
///
/// Produced by [`Perm::arrange`] or [`HandSeq::cards`]. Holds a
/// stack-allocated snapshot of the canonical cards and the decoded
/// index permutation.
///
/// Implements [`Iterator`], [`ExactSizeIterator`],
/// [`DoubleEndedIterator`], and `FusedIterator`.
///
/// [`Perm::arrange`]: super::perm::Perm::arrange
/// [`HandSeq::cards`]: super::hand_seq::HandSeq::cards
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
}

impl Iterator for CardSeq {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        (self.index < self.count).then(|| {
            let card = self.cards[self.order[self.index as usize] as usize];
            self.index += 1;
            card
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (self.count - self.index) as usize;
        (n, Some(n))
    }
}
impl DoubleEndedIterator for CardSeq {
    fn next_back(&mut self) -> Option<Card> {
        (self.index < self.count).then(|| {
            self.count -= 1;
            self.cards[self.order[self.count as usize] as usize]
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
    use super::super::perm::Perm;
    use super::*;

    #[test]
    fn exact_size() {
        let hand = Hand::try_from("2c 5d Th Qs As").unwrap();
        let mut seq = Perm::from(42).arrange(hand);
        assert_eq!(seq.len(), 5);
        seq.next();
        assert_eq!(seq.len(), 4);
        seq.next_back();
        assert_eq!(seq.len(), 3);
    }
    #[test]
    fn double_ended() {
        let hand = Hand::try_from("2c 5d Th Qs As").unwrap();
        let perm = Perm::from(77);
        let forward = perm.arrange(hand).collect::<Vec<_>>();
        let backward = perm.arrange(hand).rev().collect::<Vec<_>>();
        assert_eq!(forward.into_iter().rev().collect::<Vec<_>>(), backward);
    }
    #[test]
    fn fused() {
        let hand = Hand::try_from("7h Kd").unwrap();
        let mut seq = Perm::from(1).arrange(hand);
        seq.next();
        seq.next();
        assert_eq!(seq.next(), None);
        assert_eq!(seq.next(), None);
        assert_eq!(seq.next(), None);
    }
    #[test]
    fn collect_into_hand() {
        let hand = Hand::try_from("2c 5d Th Qs As").unwrap();
        let seq = Perm::from(42).arrange(hand);
        assert_eq!(seq.collect::<Hand>(), hand);
    }
    #[test]
    fn empty_seq() {
        let seq = Perm::identity().arrange(Hand::empty());
        assert_eq!(seq.len(), 0);
        assert_eq!(seq.collect::<Vec<_>>(), vec![]);
    }
}
