#[derive(Debug, Clone)]
pub struct Hand {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Node, //? is this useful
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
    pub fn settle(&self) -> Vec<Payout> {
        if self.head.are_all_folded() {
            self.conceded_payouts()
        } else {
            self.showdown_payouts()
        }
    }

    fn conceded_payouts(&self) -> Vec<Payout> {
        let mut payouts = self.starting_payouts();
        let winner = payouts
            .iter_mut()
            .find(|p| p.status != BetStatus::Folded)
            .unwrap();
        winner.reward = self.head.pot;
        payouts
    }
    fn showdown_payouts(&self) -> Vec<Payout> {
        let mut payouts = self.starting_payouts();
        for p in payouts.iter_mut() {
            let hand = self.cards(p.position);
            let strength = LazyEvaluator::evaluate_with_kickers(hand);
            p.strength = strength;
        }
        ShowdownMachine::settle(payouts)
    }
    fn starting_payouts(&self) -> Vec<Payout> {
        let mut payouts = self
            .head
            .seats
            .iter()
            .map(|s| self.payout(s))
            .collect::<Vec<Payout>>();
        payouts.sort_by(|a, b| self.order(a, b));
        payouts
    }
    fn payout(&self, seat: &Seat) -> Payout {
        Payout {
            reward: 0,
            risked: self.risked(seat.position),
            status: seat.status,
            position: seat.position,
            strength: FullStrength(Strength::MUCK, Kickers(vec![])), // Strength::MUCK, //? Option<Strength>
        }
    }

    fn cards(&self, position: usize) -> Vec<&Card> {
        let seat = self.head.seat(position);
        let hole = &seat.hole;
        let slice_hole = &hole.cards[..];
        let slice_board = &self.head.board.cards[..];
        slice_hole
            .iter()
            .chain(slice_board.iter())
            .collect::<Vec<&Card>>()
    }
    fn risked(&self, position: usize) -> u32 {
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
    fn priority(&self, position: usize) -> u32 {
        (position.wrapping_sub(self.head.dealer).wrapping_sub(1) % self.head.seats.len()) as u32
    }
    fn order(&self, a: &Payout, b: &Payout) -> std::cmp::Ordering {
        let x = self.priority(a.position);
        let y = self.priority(b.position);
        x.cmp(&y)
    }
}

use super::payout::Payout;
use super::seat::{BetStatus, Seat};
use super::showdown::ShowdownMachine;
use super::{action::Action, node::Node};
use crate::cards::{card::Card, deck::Deck};
use crate::evaluation::evaluation::{Evaluator, LazyEvaluator};
use crate::evaluation::strength::{FullStrength, Kickers, Strength};
