use crate::*;

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
    pub fn perm(&self) -> Lehmer {
        self.0.perm()
    }

    /// The board as it stood on an earlier street, in the order it was
    /// dealt. Asking for a later street than this board has reached is not
    /// an error; it yields the board as it is.
    pub fn upto(&self, street: Street) -> Self {
        Self::from(self.cards().take(street.n_board()).collect::<Vec<Card>>().as_slice())
    }

    /// The board with its most recent street taken back — what undoing one
    /// deal leaves behind.
    pub fn undeal(&self) -> Self {
        self.upto(self.street().prev())
    }

    /// Whether this board continues `earlier`: the same cards in the same
    /// order, for as far as `earlier` runs. A board continues itself.
    pub fn extends(&self, earlier: &Self) -> bool {
        earlier.street() <= self.street() && earlier.cards().zip(self.cards()).all(|(a, b)| a == b)
    }
}

/// Cards in the order they were dealt. `Hand` is a set and loses that
/// order, so a board built through `Hand` arrives sorted by rank rather
/// than as it fell — which is the whole reason [`Board`] wraps a sequence.
impl From<&[Card]> for Board {
    fn from(cards: &[Card]) -> Self {
        let mut board = Self::empty();
        board.deal(cards);
        board
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

// SQL codec via the packed i64 representation.
#[cfg(feature = "sql")]
pokerkit::codec!(Board as i64, |b: &Board| u64::from(Hand::from(*b)) as i64, |v| Board::from(Hand::from(v as u64)));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board() {
        let board = Board::empty();
        assert_eq!(board.cards().count(), 0);
        assert_eq!(board.perm(), Lehmer::identity());
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
    fn upto_keeps_deal_order() {
        let board = Board::from(Card::parse("Qh 5d Td Ah 2c").unwrap().as_slice());
        assert_eq!(board.upto(Street::Flop).to_string(), "Qh 5d Td");
        assert_eq!(board.upto(Street::Turn).to_string(), "Qh 5d Td Ah");
        assert_eq!(board.upto(Street::Pref).to_string(), "");
    }

    #[test]
    fn upto_a_street_not_reached_is_the_board_itself() {
        let board = Board::from(Card::parse("Qh 5d Td").unwrap().as_slice());
        assert_eq!(board.upto(Street::Rive), board);
    }

    #[test]
    fn undeal_takes_back_one_street() {
        let board = Board::from(Card::parse("Qh 5d Td Ah 2c").unwrap().as_slice());
        assert_eq!(board.undeal().to_string(), "Qh 5d Td Ah");
        assert_eq!(board.undeal().undeal().to_string(), "Qh 5d Td");
        assert_eq!(board.undeal().undeal().undeal(), Board::empty());
    }

    #[test]
    fn a_board_extends_its_own_prefixes() {
        let board = Board::from(Card::parse("Qh 5d Td Ah 2c").unwrap().as_slice());
        assert!(board.extends(&board));
        assert!(board.extends(&board.undeal()));
        assert!(board.extends(&Board::empty()));
        assert!(!board.undeal().extends(&board));
    }

    #[test]
    fn a_board_does_not_extend_a_different_run() {
        let board = Board::from(Card::parse("Qh 5d Td Ah").unwrap().as_slice());
        let other = Board::from(Card::parse("Qh 5d 2s").unwrap().as_slice());
        assert!(!board.extends(&other));
    }

    #[test]
    fn hand_round_trip() {
        let hand = Hand::try_from("Qh 5d Td Ah 2c").unwrap();
        let board = Board::from(hand);
        assert_eq!(Hand::from(board), hand);
    }
}
