use super::card::Card;
use super::hand::Hand;
use super::street::Street;

/// The community cards visible to all players.
///
/// A board contains 0, 3, 4, or 5 cards corresponding to preflop, flop, turn,
/// and river respectively. Cards are added incrementally as streets progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board(Hand);

impl Board {
    /// Creates an empty board (preflop state).
    pub fn empty() -> Self {
        Self(Hand::empty())
    }
    /// Adds cards to the board. Panics if cards overlap with existing board.
    pub fn add(&mut self, hand: Hand) {
        self.0 = Hand::add(self.0, hand);
    }
    /// Resets the board to empty for a new hand.
    pub fn clear(&mut self) {
        self.0 = Hand::empty();
    }
    /// Infers the current street from board size.
    pub fn street(&self) -> Street {
        Street::from(2 + self.0.size())
    }
}

/// Board isomorphism
/// Board -> Hand is infallible
/// Hand -> Board should select at 0, 3, 4, 5 cards
impl From<Hand> for Board {
    fn from(hand: Hand) -> Self {
        Self(hand)
    }
}
impl From<Board> for Hand {
    fn from(board: Board) -> Self {
        debug_assert!(board.0.size() != 1);
        debug_assert!(board.0.size() != 2);
        debug_assert!(board.0.size() <= 5);
        board.0
    }
}
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Vec::<Card>::from(self.0)
                .into_iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
