struct Hand {
    pub cards: [Card; 7],
}

impl Hand {
    pub fn new(board: &Board, hole: &Hole) -> Hand {
        Hand {
            cards: [
                hole.cards[0].clone(),
                hole.cards[1].clone(),
                board.cards[0].clone(),
                board.cards[1].clone(),
                board.cards[2].clone(),
                board.cards[3].clone(),
                board.cards[4].clone(),
            ],
        }
    }

    fn to_bits(&self) -> u64 {
        self.cards
            .iter()
            .map(|card| card.to_int())
            .fold(0, |bits, i| bits | 1 << i)
    }
}

pub fn evaluate(board: &Board, hole: &Hole) -> u32 {
    let hand = Hand::new(board, hole);
    let bits = hand.to_bits();
    todo!()
}

use crate::cards::board::Board;
use crate::cards::card::Card;
use crate::cards::hole::Hole;
