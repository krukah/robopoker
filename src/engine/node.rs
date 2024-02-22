#[derive(Debug, Clone)]
pub struct Node {
    pub board: Board,
    pub seats: Vec<Seat>,
    pub pot: u32,
    pub dealer: usize,
    pub pointer: usize,
    pub counter: usize,
} // this data struct reads like a poem

impl Node {
    pub fn new(seats: Vec<Seat>) -> Node {
        Node {
            board: Board::new(),
            seats,
            pot: 0,
            dealer: 0,
            pointer: 0,
            counter: 0,
        }
    }

    pub fn is_end_of_hand(&self) -> bool {
        self.has_all_folded() || (self.is_end_of_street() && self.board.street == Street::River)
    }

    pub fn is_end_of_street(&self) -> bool {
        self.has_all_folded() || (self.has_all_acted() && self.has_all_matched())
    }

    pub fn next_seat(&mut self) -> Option<&Seat> {
        loop {
            if self.is_end_of_street() {
                return None;
            }
            self.counter += 1;
            self.pointer = self.after(self.pointer);
            let seat = &self.seats[self.pointer];
            match seat.status {
                BetStatus::Folded | BetStatus::Shoved => continue,
                BetStatus::Betting => return Some(seat),
            }
        }
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
        self.seats
            .iter_mut()
            .filter(|s| s.status != BetStatus::Shoved)
            .for_each(|s| s.sunk = 0);
    }

    pub fn next_hand(&mut self) {
        // -> Payout
        self.dealer = self.after(self.dealer);
        self.pointer = self.dealer;
        self.pot = 0;
        self.counter = 0;
        self.board.cards.clear();
        self.seats
            .iter_mut()
            .for_each(|s| s.status = BetStatus::Betting);
    }

    pub fn apply(&mut self, action: Action) -> Action {
        match action {
            Action::Call(seat, amount)
            | Action::Open(seat, amount)
            | Action::Raise(seat, amount)
            | Action::Shove(seat, amount) => self.wager(seat, amount),
            _ => (),
        }
        match action {
            Action::Fold(seat) => seat.status = BetStatus::Folded,
            Action::Shove(seat, _) => seat.status = BetStatus::Shoved,
            _ => (),
        }
        action
    }

    pub fn revert(&mut self, action: Action) -> Action {
        match action {
            Action::Call(seat, amount)
            | Action::Open(seat, amount)
            | Action::Raise(seat, amount)
            | Action::Shove(seat, amount) => self.wager(seat, amount),
            // should be negative but we're unsigned. will worry ab it if i ever use this fn
            _ => (),
        }
        match action {
            Action::Fold(seat) | Action::Shove(seat, _) => seat.status = BetStatus::Betting,
            _ => (),
        }
        action
    }

    fn has_all_acted(&self) -> bool {
        self.counter > self.seats.len()
    }

    fn has_all_matched(&self) -> bool {
        let bet = self.seats.iter().map(|s| s.sunk).max().unwrap_or(0);
        self.seats
            .iter()
            .filter(|s| s.status == BetStatus::Betting)
            .all(|s| s.sunk == bet)
    }

    fn has_all_folded(&self) -> bool {
        1 == self
            .seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .count()
    }

    fn after(&self, i: usize) -> usize {
        (i + 1) % self.seats.len()
    }

    fn wager(&mut self, seat: &mut Seat, amount: u32) {
        self.pot += amount;
        seat.sunk += amount;
        seat.stack -= amount;
    }
}

use super::{
    action::Action,
    seat::{BetStatus, Seat},
};
use crate::cards::board::{Board, Street};
