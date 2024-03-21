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
            self.begin_hand();
            while self.has_streets() {
                self.begin_street();
                while self.has_turns() {
                    self.end_turn();
                }
                self.end_street();
            }
            self.end_hand();
        }
    }

    pub fn gain_seat(&mut self, stack: u32, actor: Rc<dyn Player>) {
        self.hand.head.gain_seat(stack, actor);
    }
    pub fn drop_seat(&mut self, position: usize) {
        self.hand.head.drop_seat(position);
    }

    fn begin_street(&mut self) {
        self.hand.next_street();
    }
    fn begin_hand(&mut self) {
        println!("\n{}\nHAND   {}", "-".repeat(21), self.n_hands);
        self.hand.start();
    }

    fn end_turn(&mut self) {
        let seat = self.hand.head.seat_up_next();
        let action = seat.actor.act(seat, &self.hand);
        self.hand.apply(action);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    fn end_street(&mut self) {
        self.hand.head.end_street();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    fn end_hand(&mut self) {
        println!("   {}", self.hand.head.board);
        self.n_hands += 1;
        self.hand.end();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    fn has_turns(&self) -> bool {
        self.hand.head.has_more_players()
    }
    fn has_streets(&self) -> bool {
        self.hand.head.has_more_streets()
    }
    fn has_hands(&self) -> bool {
        self.hand.head.has_more_hands() // && self.n_hands < 500000
    }
}

use super::{hand::Hand, player::Player};
use std::rc::Rc;
