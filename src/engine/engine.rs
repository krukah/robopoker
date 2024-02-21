use super::{hand::Hand, player::Player};
use crate::{
    cards::{board::Street, deck::Deck},
    evaluation::evaluation::Evaluator,
};

pub struct Engine {
    hand: Hand,
    eval: Evaluator,
    players: Vec<Player>,
}

impl Engine {
    pub fn new() -> Engine {
        todo!()
    }

    pub fn add(&mut self, player: Player) {
        self.players.push(player);
    }

    pub fn remove(&mut self, player: Player) {
        self.players.retain(|p| p.index != player.index);
    }

    pub fn run(&mut self) {
        loop {
            match self.hand.node.board.street {
                Street::Pre => self.pre(),
                Street::Flop => self.flop(),
                Street::Turn => self.turn(),
                Street::River => self.river(),
            }
            if self.hand.node.is_terminal() {
                self.payout();
                self.hand.reset();
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
    }

    fn post_blinds(&mut self) {
        todo!()
    }

    fn payout(&mut self) {
        let scores: Vec<u32> = self
            .players
            .iter()
            .map(|p| &p.hand)
            .map(|h| Evaluator::evaluate(&self.hand.node.board, &h))
            .collect();
    }

    fn reset(&mut self) {
        self.hand = Hand::new();
    }
}
