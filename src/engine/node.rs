#[derive(Debug, Clone)]
pub struct Node {
    pub board: Board,     // table
    pub seats: Vec<Seat>, // rotation
    pub pot: u32,         // table
    pub dealer: usize,    // rotation
    pub counter: usize,   // rotation
    pub pointer: usize,   // rotation
} // this data struct reads like a poem

impl Node {
    pub fn new() -> Self {
        Node {
            board: Board::new(),
            seats: Vec::with_capacity(10),
            pot: 0,
            dealer: 0,
            counter: 0,
            pointer: 0,
        }
    }

    pub fn does_end_hand(&self) -> bool {
        self.are_all_folded() || (self.does_end_street() && self.board.street == Street::River)
    }
    pub fn does_end_street(&self) -> bool {
        self.are_all_folded() || self.are_all_called() || self.are_all_shoved()
    }

    pub fn next(&mut self) -> Option<&Seat> {
        'left: loop {
            if self.does_end_street() {
                return None;
            }
            match self.left().status {
                BetStatus::Playing => {
                    self.increment();
                    return Some(self.seat());
                }
                BetStatus::Folded | BetStatus::Shoved => {
                    self.increment();
                    continue 'left;
                }
            }
        }
    }
    pub fn seat(&self) -> &Seat {
        self.seats.get(self.pointer).unwrap()
    }
    pub fn left(&self) -> &Seat {
        self.seats.get(self.after(self.pointer)).unwrap()
    }
    pub fn after(&self, i: usize) -> usize {
        (i + 1) % self.seats.len()
    }

    pub fn table_stack(&self) -> u32 {
        let mut totals: Vec<u32> = self.seats.iter().map(|s| s.stack + s.stuck).collect();
        totals.sort();
        totals.pop().unwrap_or(0);
        totals.pop().unwrap_or(0)
    }
    pub fn table_stuck(&self) -> u32 {
        self.seats.iter().map(|s| s.stuck).max().unwrap()
    }

    fn are_all_folded(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .count()
            == 1
    }
    fn are_all_shoved(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .all(|s| s.status == BetStatus::Shoved)
    }
    fn are_all_called(&self) -> bool {
        let bet = self.table_stuck();
        let has_no_decision = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .count()
            == 1
            && self.counter == 0;
        let has_all_decided = self.counter >= self.seats.len();
        let has_all_matched = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .all(|s| s.stuck == bet);
        (has_all_decided || has_no_decision) && has_all_matched
    }

    pub fn next_street(&mut self) {
        self.counter = 0;
        self.pointer = self.dealer;
        self.board.street = match self.board.street {
            Street::Pre => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Pre,
        };
        for seat in &mut self.seats {
            seat.stuck = 0;
        }
        println!("  {:?}", self.board.street);
    }
    pub fn next_hand(&mut self) {
        self.pot = 0;
        self.dealer = self.after(self.dealer);
        self.counter = 0;
        self.pointer = self.dealer;
        self.board.street = Street::Pre;
        self.board.cards.clear();
        for seat in &mut self.seats {
            seat.status = BetStatus::Playing;
            seat.stuck = 0;
        }
        println!("NEXT HAND\n");
    }

    pub fn apply(&mut self, action: Action) {
        let seat = self.seats.get_mut(self.pointer).unwrap();
        match action {
            // modify board or player status
            Action::Fold => seat.status = BetStatus::Folded,
            Action::Shove(_) => seat.status = BetStatus::Shoved,
            Action::Draw(card) => self.board.push(card.clone()),
            _ => (),
        }
        match action {
            // modify seat and pot balances
            Action::Blind(bet) | Action::Call(bet) | Action::Raise(bet) | Action::Shove(bet) => {
                self.pot += bet;
                seat.stuck += bet;
                seat.stack -= bet;
            }
            _ => (),
        }
        match action {
            // log
            Action::Draw(_) => (),
            _ => println!("  {} {}", seat.id, action),
        }
    }
    fn increment(&mut self) {
        self.counter += 1;
        self.pointer = self.after(self.pointer);
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "\nPot:   {}", self.pot)?;
        write!(f, "\nBoard: ")?;
        for card in &self.board.cards {
            write!(f, "{}  ", card)?;
        }
        for seat in &self.seats {
            write!(f, "{}  ", seat)?;
        }
        write!(f, "\n")
    }
}

use super::{
    action::Action,
    seat::{BetStatus, Seat},
};
use crate::cards::board::{Board, Street};
use std::fmt::{Display, Formatter, Result};
