use crate::cards::board::Board;
use crate::cards::card::Card;
use crate::cards::hand::Hand;

struct Deal(Vec<Card>);

impl From<Deal> for Board {
    fn from(deal: Deal) -> Board {
        Board::from(Hand::from(deal))
    }
}

impl From<Deal> for Hand {
    fn from(deal: Deal) -> Hand {
        Hand::from(deal.0)
    }
}
