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
    pub fn settlement(&self) -> Vec<Payout> {
        if self.head.are_all_folded() {
            self.conceded_payouts()
        } else {
            self.showdown_payouts()
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
            strength: Strength::new(BestHand::MUCK, Kickers(Vec::new())),
        }
    }

    pub fn min_raise(&self) -> u32 {
        let mut stakes = self
            .head
            .seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .map(|s| s.stake)
            .collect::<Vec<u32>>();
        stakes.sort_unstable();
        let last = stakes.pop().unwrap_or(0);
        let prev = stakes.pop().unwrap_or(0);
        let diff = last - prev;
        std::cmp::max(last + diff, last + self.bblind)
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

// mutable implementation reserved for engine or solver most likeliy

impl Hand {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Draw(_) => (),
            _ => println!("{action}"),
        }
        self.actions.push(action.clone());
        self.head.apply(action.clone());
    }
    pub fn start(&mut self) {
        self.actions.clear();
        self.head.start_hand();
        self.tail = self.head.clone();
        self.post_blind(self.sblind);
        self.post_blind(self.bblind);
        self.head.counter = 0;
        self.deck = Deck::new();
    }

    pub fn start_street(&mut self) {
        self.head.start_street();
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
            }
            Street::Turn => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
            }
            Street::River => {
                let card = self.deck.draw().unwrap();
                self.apply(Action::Draw(card));
            }
            Street::Showdown => unreachable!(),
        }
    }
    pub fn end(&mut self) {
        for payout in self.settlement() {
            let seat = self.head.seat_mut(payout.position);
            println!("{}{}", seat, payout);
            seat.stack += payout.reward;
        }
    }
    pub fn post_blind(&mut self, size: u32) {
        let position = self.head.next().position;
        let seat = self.head.seat_mut(position);
        let bet = std::cmp::min(size, seat.stack);
        if seat.stack <= bet {
            seat.status = BetStatus::Shoved;
        }
        self.apply(Action::Blind(position, bet));
    }
}
use super::payout::Payout;
use super::seat::{BetStatus, Seat};
use super::showdown::ShowdownMachine;
use super::{action::Action, node::Node};
use crate::cards::board::Street;
use crate::cards::{card::Card, deck::Deck};
use crate::evaluation::evaluation::{Evaluator, LazyEvaluator};
use crate::evaluation::strength::{BestHand, Kickers, Strength};
