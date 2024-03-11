pub struct Engine {
    players: Vec<Player>,
    n_hands: u32,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            players: Vec::with_capacity(10),
            n_hands: 0,
        }
    }

    pub fn play(&mut self, hand: &mut Hand) {
        'hands: loop {
            if self.has_exhausted_hands(hand) {
                break 'hands;
            }
            self.start_hand(hand);
            'streets: loop {
                if self.has_exhausted_streets(hand) {
                    break 'streets;
                }
                self.start_street(hand);
                'turns: loop {
                    if self.has_exhausted_turns(hand) {
                        break 'turns;
                    }
                    self.end_turn(hand);
                }
                self.end_street(hand);
            }
            self.end_hand(hand);
        }
    }

    fn start_street(&self, hand: &mut Hand) {
        hand.head.beg_street();
    }
    fn start_hand(&self, hand: &mut Hand) {
        println!("HAND  {}\n", self.n_hands);
        hand.beg_hand();
        hand.head.beg_hand();
        hand.post_blinds();
        hand.deal_holes();
    }

    fn end_turn(&self, hand: &mut Hand) {
        let seat = hand.head.next();
        let action = seat.player.act(seat, hand);
        hand.apply(action);
    }
    fn end_street(&self, hand: &mut Hand) {
        hand.head.end_street();
        hand.deal_board();
    }
    fn end_hand(&mut self, hand: &mut Hand) {
        self.n_hands += 1;
        hand.settle();
        println!("{}", hand.head);
    }

    fn has_exhausted_turns(&self, hand: &Hand) -> bool {
        !hand.head.has_more_players()
    }
    fn has_exhausted_streets(&self, hand: &Hand) -> bool {
        !hand.head.has_more_streets()
    }
    fn has_exhausted_hands(&self, hand: &Hand) -> bool {
        !hand.head.has_more_hands() || self.n_hands > 10000
    }
}

use super::{game::Hand, player::Player};
