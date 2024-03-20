pub struct Engine {
    hand: Hand,
    n_hands: u32,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            hand: Hand::new(),
            n_hands: 0,
        }
    }

    pub fn start(&mut self) {
        while self.has_hands() {
            self.start_hand();
            while self.has_streets() {
                self.start_street();
                while self.has_turns() {
                    self.take_turn();
                }
                self.end_street();
            }
            self.end_hand();
        }
    }

    pub fn gain_seat(&mut self, stack: u32, actor: Rc<dyn Player>) {
        println!("ADD  {}", self.hand.head.seats.len());
        let seat = Seat::new(stack, self.hand.head.seats.len(), actor);
        self.hand.head.seats.push(seat);
    }

    pub fn drop_seat(&mut self, position: usize) {
        println!("DROP {}", position);
        self.hand.head.seats.retain(|s| s.position != position);
    }

    fn start_street(&mut self) {
        self.hand.start_street();
        match self.hand.head.board.street {
            Street::Pre => (),
            _ => print!("   {}", self.hand.head.board),
        }
    }
    fn start_hand(&mut self) {
        println!("\n{}\nHAND   {}", "-".repeat(21), self.n_hands);
        self.hand.start();
    }
    fn take_turn(&mut self) {
        let seat = self.hand.head.next();
        let action = seat.actor.act(seat, &self.hand);
        self.hand.apply(action);
    }
    fn end_street(&mut self) {
        self.hand.head.end_street();
    }
    fn end_hand(&mut self) {
        print!("{}\n   {}", "-".repeat(21), self.hand.head.board);
        self.n_hands += 1;
        self.hand.end();
    }

    fn has_turns(&self) -> bool {
        self.hand.head.has_more_players()
    }
    fn has_streets(&self) -> bool {
        self.hand.head.has_more_streets()
    }
    fn has_hands(&self) -> bool {
        self.n_hands < 5000
    }
}

use super::{hand::Hand, player::Player, seat::Seat};
use crate::cards::board::Street;
use std::rc::Rc;
