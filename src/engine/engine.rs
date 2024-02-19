use crate::cards::board::Street;

use super::hand::Hand;

pub struct Engine {
    hand: Hand,
}

impl Engine {
    pub fn new() -> Engine {
        todo!()
    }

    fn start(&self) {
        loop {
            match self.hand.node.board.street {
                Street::Pre => self.pre(),
                Street::Flop => self.flop(),
                Street::Turn => self.turn(),
                Street::River => self.river(),
                Street::Showdown => break self.showdown(),
            }
        }
    }

    fn pre(&self) {
        // deal hole cards
        // collect blinds
        // first betting round
    }

    fn flop(&self) {
        // deal flop
        // second betting round
    }

    fn turn(&self) {
        // deal turn
        // third betting round
    }

    fn river(&self) {
        // deal river
        // fourth betting round
    }

    fn showdown(&self) {
        // compare hands
        // award pot
    }
}
