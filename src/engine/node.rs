#[derive(Debug, Clone)]
pub struct Node {
    pub board: Board,     // table
    pub seats: Vec<Seat>, // rotation
    pub pot: u32,         // table
    pub dealer: usize,    // rotation
    pub counter: usize,   // rotation
    pub pointer: usize,   // rotation.has_next == node.does_end_street
} // this data struct reads like a poem

impl Node {
    pub fn new(seats: Vec<Seat>) -> Self {
        Node {
            seats,
            board: Board::new(),
            pot: 0,
            dealer: 0,
            counter: 0,
            pointer: 0,
        }
    }

    pub fn has_more_hands(&self) -> bool {
        self.seats.iter().filter(|s| s.stack > 0).count() > 1
    }
    pub fn has_more_streets(&self) -> bool {
        !(self.are_all_folded() || (!self.has_more_players() && self.board.street == Street::River))
    }
    pub fn has_more_players(&self) -> bool {
        !(self.are_all_folded() || self.are_all_called() || self.are_all_shoved())
    }

    pub fn begin_hand(&mut self) {
        self.pot = 0;
        self.board.cards.clear();
        self.board.street = Street::Pre;
        self.counter = 0;
        self.dealer = self.after(self.dealer);
        self.pointer = self.dealer;
        self.advance();
    }
    pub fn begin_street(&mut self) {
        self.counter = 0;
        self.pointer = match self.board.street {
            Street::Pre => self.after(self.after(self.dealer)),
            _ => self.dealer,
        };
        self.advance();
    }
    pub fn apply(&mut self, action: Action) {
        let seat = self.seats.get_mut(self.pointer).unwrap();
        match action {
            Action::Draw(_) => (),
            _ => println!("{action}"),
        }
        match action {
            Action::Call(_, bet)
            | Action::Blind(_, bet)
            | Action::Raise(_, bet)
            | Action::Shove(_, bet) => {
                self.pot += bet;
                seat.stuck += bet;
                seat.stack -= bet;
            }
            _ => (),
        }
        match action {
            Action::Fold(..) => seat.status = BetStatus::Folded,
            Action::Shove(..) => seat.status = BetStatus::Shoved,
            _ => (),
        }
        match action {
            Action::Draw(card) => self.board.push(card.clone()),
            _ => self.advance(),
        }
    }

    pub fn next(&self) -> &Seat {
        self.seats.get(self.pointer).unwrap()
    }
    pub fn seat(&self, id: usize) -> &Seat {
        self.seats.iter().find(|s| s.id == id).unwrap()
    }
    pub fn after(&self, index: usize) -> usize {
        (index + 1) % self.seats.len()
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

    fn advance(&mut self) {
        'left: loop {
            if self.has_more_players() {
                self.counter += 1;
                self.pointer = self.after(self.pointer);
                match self.next().status {
                    BetStatus::Playing => return,
                    BetStatus::Folded | BetStatus::Shoved => continue 'left,
                }
            }
            return;
        }
    }

    pub fn are_all_folded(&self) -> bool {
        // exactly one player has not folded
        self.seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .count()
            == 1
    }
    pub fn are_all_shoved(&self) -> bool {
        // everyone who isn't folded is all in
        self.seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .all(|s| s.status == BetStatus::Shoved)
    }
    pub fn are_all_called(&self) -> bool {
        // everyone who isn't folded has matched the bet
        // or all but one player is all in
        let bet = self.table_stuck();
        let is_one_playing = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .count()
            == 1;
        let is_first_decision = self.counter == 0;
        let has_no_decision = is_first_decision && is_one_playing;
        let has_all_decided = self.counter > self.seats.len();
        let has_all_matched = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .all(|s| s.stuck == bet);
        (has_all_decided || has_no_decision) && has_all_matched
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
