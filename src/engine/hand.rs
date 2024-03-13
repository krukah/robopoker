#[derive(Debug, Clone)]
pub struct Hand {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Node, // is this useful?
    pub head: Node,
    pub outcomes: Vec<HandResult>,
    pub actions: Vec<Action>,
}
impl Hand {
    pub fn new() -> Self {
        Hand {
            sblind: 1,
            bblind: 2,
            actions: Vec::new(),
            outcomes: Vec::new(),
            deck: Deck::new(),
            tail: Node::new(),
            head: Node::new(),
        }
    }

    pub fn apply(&mut self, action: Action) {
        self.head.apply(action.clone());
        self.actions.push(action.clone());
        match action {
            Action::Draw(_) => (),
            _ => println!("{action}"),
        }
    }

    pub fn reset_hand(&mut self) {
        for seat in self.head.seats.iter_mut() {
            seat.status = BetStatus::Playing;
            seat.stake = 0;
        }
        self.tail = self.head.clone();
        self.deck = Deck::new();
        self.outcomes.clear();
        self.actions.clear();
    }

    pub fn settle(&mut self) {
        for result in self.showdown().results() {
            let seat = self
                .head
                .seats
                .iter_mut()
                .find(|s| s.seat_id == result.seat_id)
                .unwrap();
            seat.stack += result.reward;
        }
        println!("{}", self.head);
    }

    pub fn post_blinds(&mut self) {
        self.apply(Action::Blind(self.head.to_act().seat_id, self.sblind));
        self.apply(Action::Blind(self.head.to_act().seat_id, self.bblind));
        self.head.counter = 0;
    }

    pub fn deal_holes(&mut self) {
        for hole in self.head.seats.iter_mut().map(|s| &mut s.hole) {
            hole.cards.clear();
            hole.cards.push(self.deck.draw().unwrap());
            hole.cards.push(self.deck.draw().unwrap());
        }
    }

    pub fn deal(&mut self) {
        match self.head.board.street {
            Street::Pre => {
                for hole in self.head.seats.iter_mut().map(|s| &mut s.hole) {
                    hole.cards.clear();
                    hole.cards.push(self.deck.draw().unwrap());
                    hole.cards.push(self.deck.draw().unwrap());
                }
            }
            Street::Flop => {
                let card1 = self.deck.draw().unwrap();
                let card2 = self.deck.draw().unwrap();
                let card3 = self.deck.draw().unwrap();
                self.apply(Action::Draw(card1));
                self.apply(Action::Draw(card2));
                self.apply(Action::Draw(card3));
                println!("FLOP   {} {} {}", card1, card2, card3);
            }
            Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
                println!("TURN   {}", card)
            }
            Street::River => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
                println!("RIVER  {}", card)
            }
            Street::Showdown => unreachable!(),
        }
    }

    pub fn advance_street(&mut self) {
        self.head.board.street = match self.head.board.street {
            Street::Pre => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Showdown,
            Street::Showdown => unreachable!(),
        }
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
    fn priority(&self, id: usize) -> u32 {
        // TODO: misuse of ID as position
        (id.wrapping_sub(self.head.dealer).wrapping_sub(1) % self.head.seats.len()) as u32
    }
}

use super::{action::Action, node::Node, payoff::HandResult, seat::BetStatus, showdown::Showdown};
use crate::cards::{board::Street, deck::Deck};
use rand::Rng;
