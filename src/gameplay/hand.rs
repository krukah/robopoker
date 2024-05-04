#[derive(Debug, Clone)]
pub struct Hand {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Rotation,
    pub head: Rotation,
    pub actions: Vec<Action>,
}
#[allow(dead_code)]
impl Hand {
    pub fn new() -> Self {
        Hand {
            sblind: 1,
            bblind: 2,
            actions: Vec::new(),
            deck: Deck::new(),
            tail: Rotation::new(),
            head: Rotation::new(),
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

    fn street_bets(&self, street: Street) -> Vec<Action> {
        let edges = self.street_bounds();
        let range = self.street_range(street, edges);
        self.actions[range].to_vec()
    }
    fn street_bounds(&self) -> Vec<usize> {
        let mut n_draws = 0usize;
        let mut boundaries = Vec::new();
        self.actions
            .iter()
            .enumerate()
            .filter(|(_, a)| match a {
                Action::Draw(..) => true,
                _ => false,
            })
            .for_each(|(i, _)| {
                n_draws += 1;
                if n_draws >= 3 {
                    boundaries.push(i);
                }
            });
        boundaries
    }
    fn street_range(&self, street: Street, bounds: Vec<usize>) -> std::ops::Range<usize> {
        match street {
            Street::Pre => 0..bounds[0],
            Street::Flop => bounds[0]..bounds[1],
            Street::Turn => bounds[1]..bounds[2],
            Street::River => bounds[2]..self.actions.len(),
            Street::Showdown => unreachable!(),
        }
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
            risked: self.risked(seat.position()),
            status: seat.status(),
            position: seat.position(),
            strength: LazyEvaluator::strength(self.cards(seat.position())),
        }
    }

    pub fn min_raise(&self) -> u32 {
        let mut stakes = self
            .head
            .seats
            .iter()
            .filter(|s| s.status() != BetStatus::Folded)
            .map(|s| s.stake())
            .collect::<Vec<u32>>();
        stakes.sort_unstable();
        let last = stakes.pop().unwrap_or(0);
        let prev = stakes.pop().unwrap_or(0);
        let diff = last - prev;
        std::cmp::max(last + diff, last + self.bblind)
    }
    fn cards(&self, position: usize) -> Vec<&Card> {
        let seat = self.head.seat_at_position(position);
        let hole = seat.peek();
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
    fn priority(&self, position: usize) -> usize {
        (self.head.seats.len() + position - self.head.after(self.head.dealer))
            % self.head.seats.len()
    }
    fn order(&self, a: &Payout, b: &Payout) -> std::cmp::Ordering {
        let x = self.priority(a.position);
        let y = self.priority(b.position);
        x.cmp(&y)
    }
}

// mutable implementation reserved for engine or solver
impl Hand {
    pub fn apply(&mut self, action: Action) {
        self.actions.push(action);
        self.head.apply(action);
    }
    pub fn start(&mut self) {
        self.head.begin_hand();
        self.tail = self.head.clone();
        self.actions.clear();
        self.post(self.sblind);
        self.post(self.bblind);
        self.head.counter = 0;
        self.deck = Deck::new();
    }
    pub fn post(&mut self, size: u32) {
        let pointer = self.head.pointer;
        let seat = self.head.seat_at_position_mut(pointer);
        let bet = std::cmp::min(size, seat.stack());
        if seat.stack() <= bet {
            seat.set_status(BetStatus::Shoved);
        }
        self.apply(Action::Blind(pointer, bet));
    }
    pub fn next_street(&mut self) {
        self.head.begin_street();
        match self.head.board.street {
            Street::Pre => {
                for hole in self.head.seats.iter_mut().map(|s| s.hole()) {
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
                println!("   {}", self.head.board)
            }
            Street::Turn | Street::River => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
                println!("   {}", self.head.board)
            }
            Street::Showdown => unreachable!(),
        }
    }
    pub fn end(&mut self) {
        let mut payouts = self.settlement();
        payouts.sort_by(|a, b| a.position.cmp(&b.position));
        for payout in payouts {
            let seat = self.head.seat_at_position_mut(payout.position);
            seat.win(payout.reward);
        }
        self.head.prune()
    }
}
use super::payout::Payout;
use super::seat::{BetStatus, Seat};
use super::{action::Action, rotation::Rotation};
use crate::cards::board::Street;
use crate::cards::{card::Card, deck::Deck};
use crate::evaluation::evaluation::{Evaluator, LazyEvaluator};
use crate::evaluation::showdown::Showdown;
