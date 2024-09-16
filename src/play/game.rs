#[derive(Debug, Clone)]
pub struct Game {
    //  pot: Chips,
    //  board: Board,
    head: Spot,
    history: Vec<Action>,
    //  deck: Deck,           // implied by Rotation
    //  bblind: Chips,        // const fn
    //  sblind: Chips,        // const fn
}

#[allow(dead_code)]
impl Game {
    pub fn new() -> Self {
        Game {
            history: Vec::new(),
            head: Spot::new(),
        }
    }
    pub fn settlement(&self) -> Vec<Payout> {
        let payouts = self.starting_payouts();
        if self.head.are_all_folded() {
            Showdown::concede(payouts)
        } else {
            Showdown::settle(payouts)
        }
    }

    fn starting_payouts(&self) -> Vec<Payout> {
        let mut payouts = self
            .head
            .chairs
            .iter()
            .map(|s| self.payout(s))
            .collect::<Vec<Payout>>();
        payouts.sort_by(|a, b| self.order(a, b));
        payouts
    }
    fn payout(&self, seat: &Seat) -> Payout {
        let position = seat.position();
        let cards = self.cards(position);
        Payout {
            reward: 0,
            strength: Strength::from(Hand::from(cards)),
        }
    }

    fn cards(&self, position: usize) -> Vec<Card> {
        let seat = self.head.actor_ref();
        let hole = *seat.peek();
        let hand = Hand::add(Hand::from(hole), Hand::from(self.head.board));
        Vec::<Card>::from(hand)
    }
    fn risked(&self, position: usize) -> Chips {
        self.history
            .iter()
            .filter(|a| match a {
                Action::Call(id_, _)
                | Action::Blind(id_, _)
                | Action::Raise(id_, _)
                | Action::Shove(id_, _) => *id_ == position,
                _ => false,
            })
            .map(|a| match a {
                Action::Call(_, bet)
                | Action::Blind(_, bet)
                | Action::Raise(_, bet)
                | Action::Shove(_, bet) => *bet,
                _ => 0,
            })
            .sum()
    }
}

// mutable implementation reserved for engine or solver
// impl Game {
// fn priority(&self, position: usize) -> usize {
//     (self.head.chairs.len() + position - self.head.after(self.head.dealer))
//         % self.head.chairs.len()
// }
// fn order(&self, a: &Payout, b: &Payout) -> std::cmp::Ordering {
//     let x = self.priority(a.position);
//     let y = self.priority(b.position);
//     x.cmp(&y)
// }
// pub fn start(&mut self) {
//     self.head.begin_hand();
//     self.history.clear();
//     self.post(self.sblind);
//     self.post(self.bblind);
//     self.head.counts = 0;
//     self.deck = Deck::new();
// }
// pub fn post(&mut self, size: Chips) {
//     let mut seat = self.head.actor_mut();
//     let stack = seat.stack();
//     if stack <= size {
//         seat.reset_state(Status::Shoving);
//         self.head.apply(Action::Blind(stack));
//     } else {
//         self.head.apply(Action::Blind(size));
//     }
// }
// pub fn end(&mut self) {
//     let mut payouts = self.settlement();
//     payouts.sort_by(|a, b| a.position.cmp(&b.position));
//     for payout in payouts {
//         let seat = self.head.seat_at_position_mut(payout.position);
//         seat.win(payout.reward);
//     }
//     self.head.prune()
// }

// fn street_bets(&self, street: Street) -> Vec<Action> {
//     let edges = self.street_bounds();
//     let range = self.street_range(street, edges);
//     self.actions[range].to_vec()
// }
// fn street_bounds(&self) -> Vec<usize> {
//     let mut n_draws = 0usize;
//     let mut boundaries = Vec::new();
//     self.actions
//         .iter()
//         .enumerate()
//         .filter(|(_, a)| match a {
//             Action::Draw(..) => true,
//             _ => false,
//         })
//         .for_each(|(i, _)| {
//             n_draws += 1;
//             if n_draws >= 3 {
//                 boundaries.push(i);
//             }
//         });
//     boundaries
// }
// fn street_range(&self, street: Street, bounds: Vec<usize>) -> std::ops::Range<usize> {
//     match street {
//         Street::Pref => 0..bounds[0],
//         Street::Flop => bounds[0]..bounds[1],
//         Street::Turn => bounds[1]..bounds[2],
//         Street::Rive => bounds[2]..self.actions.len(),
//         Street::Show => unreachable!(),
//     }
// }
// }
use super::payout::Payout;
use super::seat::{Seat, State};
use super::showdown::Showdown;
use super::Chips;
use super::{action::Action, rotation::Spot};
use crate::cards::hand::Hand;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
use crate::cards::{card::Card, deck::Deck};
