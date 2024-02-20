use super::hand::Hand;
use crate::cards::board::Street;

pub struct Engine {
    hand: Hand,
}

impl Engine {
    pub fn new() -> Engine {
        todo!()
    }

    fn start(&mut self) {
        loop {
            match self.hand.node.board.street {
                Street::Pre => self.pre(),
                Street::Flop => self.flop(),
                Street::Turn => self.turn(),
                Street::River => self.river(),
                Street::Showdown => self.showdown(),
            }
            if self.is_hand_complete() {
                break;
            }
        }
    }

    fn pre(&mut self) {
        // deal hole cards
        // collect blinds
        // first betting round
        self.hand.node.board.street = Street::Flop;
    }

    fn flop(&mut self) {
        // deal flop
        // second betting round
        self.hand.node.board.street = Street::Turn;
    }

    fn turn(&mut self) {
        // deal turn
        // third betting round
        self.hand.node.board.street = Street::River;
    }

    fn river(&mut self) {
        // deal river
        // fourth betting round
        self.hand.node.board.street = Street::Showdown;
    }

    fn showdown(&mut self) {
        // compare hands
        // award pot
    }

    fn post_blinds(&mut self) {
        todo!()
    }

    fn is_hand_complete(&self) -> bool {
        todo!()
    }
}
