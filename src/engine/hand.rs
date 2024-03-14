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

    pub fn results(&self) -> Vec<HandResult> {
        let showdown = self.showdown();
        showdown.results()
    }
    fn showdown(&self) -> Showdown {
        let mut results = self
            .head
            .seats
            .iter()
            .map(|s| HandResult {
                reward: 0,
                score: self.score(s.seat_id),
                staked: self.staked(s.seat_id),
                status: s.status,
                seat_id: s.seat_id,
            })
            .collect::<Vec<HandResult>>();
        results.sort_by(|a, b| {
            let x = self.priority(a.seat_id);
            let y = self.priority(b.seat_id);
            x.cmp(&y)
        });
        Showdown {
            next_stake: u32::MIN,
            prev_stake: u32::MIN,
            next_score: u32::MAX,
            results,
        }
    }
    fn staked(&self, id: usize) -> u32 {
        self.actions
            .iter()
            .filter(|a| match a {
                Action::Call(id_, _)
                | Action::Blind(id_, _)
                | Action::Raise(id_, _)
                | Action::Shove(id_, _) => *id_ == id,
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
    fn score(&self, _id: usize) -> u32 {
        rand::thread_rng().gen::<u32>() % 32
    }
    fn priority(&self, position: usize) -> u32 {
        // TODO: misuse of ID as position
        (position.wrapping_sub(self.head.dealer).wrapping_sub(1) % self.head.seats.len()) as u32
    }
}
// mutables

use super::{action::Action, node::Node, payoff::HandResult, showdown::Showdown};
use crate::cards::deck::Deck;
use rand::Rng;
