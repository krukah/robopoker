#[derive(Debug, Clone)]
pub struct Hand {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Node, // is this useful?
    pub head: Node,
    pub actions: Vec<Action>,
}
impl Hand {
    pub fn new() -> Self {
        Hand {
            sblind: 1,
            bblind: 2,
            actions: Vec::new(),
            deck: Deck::new(),
            tail: Node::new(),
            head: Node::new(),
        }
    }
    pub fn payouts(&self) -> Vec<Payout> {
        let mut payouts = self
            .head
            .seats
            .iter()
            .map(|s| Payout {
                reward: 0,
                score: self.score(s.seat_id),
                staked: self.staked(s.seat_id),
                status: s.status,
                position: s.seat_id,
            })
            .collect::<Vec<Payout>>();
        payouts.sort_by(|a, b| {
            let x = self.priority(a.position);
            let y = self.priority(b.position);
            x.cmp(&y)
        });
        Showdown::new(payouts).payouts()
    }
    pub fn staked(&self, position: usize) -> u32 {
        self.actions
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
    pub fn score(&self, position: usize) -> u32 {
        let hole = self
            .head
            .seats
            .iter()
            .find(|s| s.seat_id == position)
            .map(|s| s.cards())
            .unwrap();
        let slice_hole = &hole.cards[..];
        let slice_board = &self.head.board.cards[..];
        let slice_combined = &slice_hole
            .iter()
            .chain(slice_board.iter())
            .collect::<Vec<&Card>>();
        let eval = LazyEvaluator::new(slice_combined);
        eval.score()
    }
    pub fn priority(&self, position: usize) -> u32 {
        // TODO: misuse of ID as position
        (position.wrapping_sub(self.head.dealer).wrapping_sub(1) % self.head.seats.len()) as u32
    }
}
// mutables

use super::payout::Payout;
use super::showdown::Showdown;
use super::{action::Action, node::Node};
use crate::cards::{card::Card, deck::Deck};
use crate::evaluation::evaluation::LazyEvaluator;
