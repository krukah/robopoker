pub struct Engine {
    hand: Hand,
    eval: Evaluator,
    players: Vec<Player>,
}

impl Engine {
    pub fn new() -> Engine {
        let players = Vec::with_capacity(10);
        Engine {
            hand: Hand::new(players),
            eval: Evaluator {},
            players,
        }
    }

    pub fn add(&mut self, player: Player) {
        self.players.push(player);
    }

    pub fn remove(&mut self, player: Player) {
        self.players.retain(|p| p.index != player.index);
    }

    pub fn run(&mut self) {
        loop {
            let mut node = self.hand.head;
            if let Some(seat) = node.next_seat() {
                self.payout(&node);
                continue;
            }
            if node.is_end_of_street() {
                node.next_street();
                continue;
            }
            if node.is_end_of_hand() {
                node.next_hand();
                continue;
            }
        }
    }

    fn pre(&mut self) {
        // deal hole cards
        // collect blinds
        // first betting round
    }

    fn flop(&mut self) {
        // deal flop
        // second betting round
    }

    fn turn(&mut self) {
        // deal turn
        // third betting round
    }

    fn river(&mut self) {
        // deal river
        // fourth betting round
    }

    fn post_blinds(&mut self) {
        todo!()
    }

    fn payout(&mut self, node: &Node) {
        let scores: Vec<u32> = self
            .players
            .iter()
            .map(|p| Evaluator::evaluate(&node.board, &p.hand))
            .collect();
    }
}

use super::{
    hand::Hand,
    node::{self, Node},
    player::{self, Player},
    seat::{BetStatus, Seat},
};
use crate::{
    cards::{board::Street, deck::Deck},
    evaluation::evaluation::Evaluator,
};
