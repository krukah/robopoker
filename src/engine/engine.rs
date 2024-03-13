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

    pub fn gain_seat(&mut self, stack: u32, actor: Rc<dyn Actor>) {
        println!("ADD  {}\n", self.hand.head.seats.len());
        let seat = Seat::new(stack, self.hand.head.seats.len(), actor);
        self.hand.head.seats.push(seat);
    }

    pub fn lose_seat(&mut self, seat_id: usize) {
        println!("DROP {}\n", seat_id);
        self.hand.head.seats.retain(|s| s.seat_id != seat_id);
    }

    pub fn start(&mut self) {
        loop {
            if self.has_exhausted_hands() {
                break;
            }
            self.start_hand();
            loop {
                if self.has_exhausted_streets() {
                    break;
                }
                self.start_street();
                loop {
                    if self.has_exhausted_turns() {
                        break;
                    }
                    self.end_turn();
                }
                self.end_street();
            }
            self.end_hand();
        }
    }

    fn start_street(&mut self) {
        self.hand.head.start_street();
    }
    fn start_hand(&mut self) {
        println!("HAND  {}\n", self.n_hands);
        self.hand.beg_hand();
        self.hand.head.start_hand();
        self.hand.post_blinds();
        self.hand.deal_holes();
    }

    fn end_turn(&mut self) {
        let seat = self.hand.head.to_act();
        let action = seat.actor.act(seat, &self.hand);
        self.hand.apply(action);
    }
    fn end_street(&mut self) {
        self.hand.head.end_street();
        self.hand.deal_board();
    }
    fn end_hand(&mut self) {
        self.n_hands += 1;
        self.hand.settle();
    }

    fn has_exhausted_turns(&self) -> bool {
        !self.hand.head.has_more_players()
    }
    fn has_exhausted_streets(&self) -> bool {
        !self.hand.head.has_more_streets()
    }
    fn has_exhausted_hands(&self) -> bool {
        !self.hand.head.has_more_hands()
    }
}

use std::rc::Rc;

use super::{hand::Hand, player::Actor, seat::Seat};
