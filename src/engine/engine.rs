pub struct Engine<'a> {
    hand: &'a mut GameHand<'a>,
    players: Vec<Player>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Engine<'static> {
        // build hand from players
        // build hand from deck
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
            // take action given the seat
            while let Some(seat) = self.hand.head.next_seat() {
                todo!();
            }
            // advance to next street
            if self.hand.head.is_end_of_street() {
                todo!();
            }
            // advance to next hand
            if self.hand.head.is_end_of_hand() {
                todo!();
            }
        }
    }

    fn payout(&mut self, _: &Node) {
        todo!()
    }
}

use super::{game_hand::GameHand, node::Node, player::Player};
