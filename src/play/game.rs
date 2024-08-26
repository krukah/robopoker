#[derive(Debug, Clone)]
pub struct Game {
    pub bblind: u32,
    pub sblind: u32,
    pub deck: Deck,
    pub tail: Rotation,
    pub head: Rotation,
    pub actions: Vec<Action>,
}
#[allow(dead_code)]
impl Game {
    pub fn new() -> Self {
        Game {
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
            Street::Pref => 0..bounds[0],
            Street::Flop => bounds[0]..bounds[1],
            Street::Turn => bounds[1]..bounds[2],
            Street::Rive => bounds[2]..self.actions.len(),
            Street::Show => unreachable!(),
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
        let position = seat.position();
        let status = seat.status();
        let risked = self.risked(position);
        let cards = self.cards(position);
        Payout {
            reward: 0,
            risked,
            status,
            strength: Strength::from(Hand::from(cards)),
            position,
        }
    }

    pub fn min_raise(&self) -> u32 {
        let mut stakes = self
            .head
            .seats
            .iter()
            .filter(|s| s.status() != Status::Folding)
            .map(|s| s.stake())
            .collect::<Vec<u32>>();
        stakes.sort_unstable();
        let last = stakes.pop().unwrap_or(0);
        let prev = stakes.pop().unwrap_or(0);
        let diff = last - prev;
        std::cmp::max(last + diff, last + self.bblind)
    }
    fn cards(&self, position: usize) -> Vec<Card> {
        let seat = self.head.at(position);
        let hole = *seat.peek();
        let hand = Hand::add(Hand::from(hole), Hand::from(self.head.board));
        Vec::<Card>::from(hand)
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
impl Game {
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
        self.head.counts = 0;
        self.deck = Deck::new();
    }
    pub fn post(&mut self, size: u32) {
        let pointer = self.head.action;
        let seat = self.head.seat_at_position_mut(pointer);
        let bet = std::cmp::min(size, seat.stack());
        if seat.stack() <= bet {
            seat.set(Status::Shoving);
        }
        self.apply(Action::Blind(pointer, bet));
    }
    pub fn next_street(&mut self) {
        self.head.begin_street();
        match self.head.board.street() {
            Street::Pref => {
                for hole in self.head.seats.iter_mut().map(|s| s.hole()) {
                    self.head.board.deal(hole)
                }
            }
            Street::Flop => {
                let card1 = self.deck.flip();
                let card2 = self.deck.flip();
                let card3 = self.deck.flip();
                self.apply(Action::Draw(card1));
                self.apply(Action::Draw(card2));
                self.apply(Action::Draw(card3));
                println!("   {}", self.head.board)
            }
            Street::Turn | Street::Rive => {
                let card = self.deck.flip();
                self.apply(Action::Draw(card));
                println!("   {}", self.head.board)
            }
            Street::Show => unreachable!(),
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
use super::seat::{Status, Seat};
use super::showdown::Showdown;
use super::{action::Action, rotation::Rotation};
use crate::cards::hand::Hand;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
use crate::cards::{card::Card, deck::Deck};
