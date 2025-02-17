use super::card::Card;
use super::hand::Hand;
use super::street::Street;

#[derive(Debug, Clone, Copy)]
pub struct Board(Hand);

impl Board {
    /// create an empty board
    pub fn empty() -> Self {
        Self(Hand::empty())
    }
    /// add a card to the board
    pub fn add(&mut self, hand: Hand) {
        self.0 = Hand::add(self.0, hand);
    }
    /// clear the board
    pub fn clear(&mut self) {
        self.0 = Hand::empty();
    }
    /// what street is this board on?
    pub fn street(&self) -> Street {
        Street::from(self.0.size())
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
        assert!(board.0.size() != 1);
        assert!(board.0.size() != 2);
        assert!(board.0.size() <= 5);
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
