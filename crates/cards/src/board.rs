use super::card::Card;
use super::card_seq::CardSeq;
use super::hand::Hand;
use super::hand_seq::HandSeq;
use super::perm::Perm;
use super::street::Street;

/// The community cards visible to all players.
///
/// A board contains 0, 3, 4, or 5 cards corresponding to preflop, flop, turn,
/// and river respectively. Wraps a [`HandSeq`] to preserve deal-time ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board(HandSeq);

impl Board {
    /// Creates an empty board (preflop state).
    pub fn empty() -> Self {
        Self(HandSeq::empty())
    }
    /// Adds cards to the board in canonical (ascending) order.
    pub fn add(&mut self, hand: Hand) {
        self.0.add(hand);
    }
    /// Adds cards to the board in the given deal order.
    pub fn deal(&mut self, cards: &[Card]) {
        self.0.deal(cards);
    }
    /// Resets the board to empty for a new hand.
    pub fn clear(&mut self) {
        self.0.clear();
    }
    /// Infers the current street from board size.
    pub fn street(&self) -> Street {
        Street::from(2 + self.0.size())
    }
    /// Board cards in deal order as a zero-allocation iterator.
    pub fn cards(&self) -> CardSeq {
        self.0.cards()
    }
    /// The board's deal-order permutation.
    pub fn perm(&self) -> Perm {
        self.0.perm()
    }
}

impl From<Hand> for Board {
    fn from(hand: Hand) -> Self {
        Self(HandSeq::from(hand))
    }
}
impl From<Board> for Hand {
    fn from(board: Board) -> Self {
        debug_assert!(board.0.size() != 1);
        debug_assert!(board.0.size() != 2);
        debug_assert!(board.0.size() <= 5);
        Hand::from(board.0)
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board() {
        let board = Board::empty();
        assert_eq!(board.cards().count(), 0);
        assert_eq!(board.perm(), Perm::identity());
    }
    #[test]
    fn deal_preserves_order() {
        let mut board = Board::empty();
        board.deal(&Card::parse("Qh 5d Td").unwrap());
        assert_eq!(board.to_string(), "Qh 5d Td");
    }
    #[test]
    fn deal_incremental() {
        let mut board = Board::empty();
        board.deal(&Card::parse("Qh 5d Td").unwrap());
        board.deal(&Card::parse("Ah").unwrap());
        board.deal(&Card::parse("2c").unwrap());
        assert_eq!(board.to_string(), "Qh 5d Td Ah 2c");
        assert_eq!(board.street(), Street::Rive);
    }
    #[test]
    fn hand_round_trip() {
        let hand = Hand::try_from("Qh 5d Td Ah 2c").unwrap();
        let board = Board::from(hand);
        assert_eq!(Hand::from(board), hand);
    }
}
