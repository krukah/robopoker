use super::card::Card;
use super::hand::Hand;
use super::hole::Hole;
use super::street::Street;

#[derive(Debug, Clone, Copy)]
pub struct Board(Hand);

impl Board {
    pub fn new() -> Self {
        Self(Hand::from(0u64))
    }

    pub fn advance(&mut self) {
        todo!("need to couple with self.add")
    }

    pub fn add(&mut self, card: Card) {
        self.0 = Hand::add(self.0, Hand::from(u64::from(card)));
    }

    pub fn clear(&mut self) {
        self.0 = Hand::from(0u64);
    }

    pub fn deal(&mut self, _: &mut Hole) {
        todo!("draw from self, add to hole")
    }

    pub fn street(&self) -> Street {
        match self.0.size() {
            0 => Street::Pref,
            3 => Street::Flop,
            4 => Street::Turn,
            5 => Street::Rive,
            _ => panic!("Invalid board size"),
        }
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
        board.0
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
