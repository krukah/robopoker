#![allow(dead_code)]

use super::action::Action;
use super::seat::BetStatus;
use super::seat::Seat;
use crate::cards::board::Board;
use crate::cards::street::Street;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// Rotation represents the memoryless state of the game in between actions.
///
/// It records both public and private data structs, and is responsible for managing the
/// rotation of players, the pot, and the board. Its immutable methods reveal
/// pure functions representing the rules of how the game may proceed.
#[derive(Debug, Clone)]
pub struct Rotation {
    pub pot: u32,
    pub dealer: usize,
    pub counts: usize,
    pub action: usize,
    pub board: Board,
    pub seats: Vec<Seat>,
}

impl Rotation {
    pub fn new() -> Self {
        Rotation {
            seats: Vec::with_capacity(10),
            board: Board::new(),
            pot: 0,
            dealer: 0,
            counts: 0,
            action: 0,
        }
    }

    pub fn has_more_hands(&self) -> bool {
        self.seats.iter().filter(|s| s.stack() > 0).count() > 1
    }
    pub fn has_more_streets(&self) -> bool {
        !(self.are_all_folded() || (self.board.street == Street::Show))
    }
    pub fn has_more_players(&self) -> bool {
        !(self.are_all_folded() || self.are_all_called() || self.are_all_shoved())
    }

    pub fn seat_up_next(&self) -> &Seat {
        self.seats.get(self.action).unwrap()
    }
    pub fn seat_at_position(&self, index: usize) -> &Seat {
        self.seats.iter().find(|s| s.position() == index).unwrap()
    }
    pub fn seat_at_position_mut(&mut self, index: usize) -> &mut Seat {
        self.seats
            .iter_mut()
            .find(|s| s.position() == index)
            .unwrap()
    }
    pub fn after(&self, index: usize) -> usize {
        (index + 1) % self.seats.len()
    }
    pub fn before(&self, index: usize) -> usize {
        (index + self.seats.len() - 1) % self.seats.len()
    }

    pub fn effective_stack(&self) -> u32 {
        let mut totals = self
            .seats
            .iter()
            .map(|s| s.stack() + s.stake())
            .collect::<Vec<u32>>();
        totals.sort();
        totals.pop().unwrap_or(0);
        totals.pop().unwrap_or(0)
    }
    pub fn effective_stake(&self) -> u32 {
        self.seats.iter().map(|s| s.stake()).max().unwrap()
    }

    pub fn are_all_folded(&self) -> bool {
        // exactly one player has not folded
        self.seats
            .iter()
            .filter(|s| s.status() != BetStatus::Folded)
            .count()
            == 1
    }
    pub fn are_all_shoved(&self) -> bool {
        // everyone who isn't folded is all in
        self.seats
            .iter()
            .filter(|s| s.status() != BetStatus::Folded)
            .all(|s| s.status() == BetStatus::Shoved)
    }
    pub fn are_all_called(&self) -> bool {
        // everyone who isn't folded has matched the bet
        // or all but one player is all in
        let stakes = self.effective_stake();
        let is_first_decision = self.counts == 0;
        let is_one_playing = self
            .seats
            .iter()
            .filter(|s| s.status() == BetStatus::Playing)
            .count()
            == 1;
        let has_no_decision = is_first_decision && is_one_playing;
        let has_all_decided = self.counts > self.seats.len();
        let has_all_matched = self
            .seats
            .iter()
            .filter(|s| s.status() == BetStatus::Playing)
            .all(|s| s.stake() == stakes);
        (has_all_decided || has_no_decision) && has_all_matched
    }
}

impl Display for Rotation {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "Pot:   {}", self.pot)?;
        writeln!(f, "Board: {}", self.board)?;
        for seat in &self.seats {
            write!(f, "{}", seat)?;
        }
        Ok(())
    }
}

// mutable methods are lowkey reserved for the node's owning Hand -- maybe some CFR engine

impl Rotation {
    pub fn apply(&mut self, action: Action) {
        let seat = self.seats.get_mut(self.action).unwrap();
        // bets entail pot and stack change
        // folds and all-ins entail status change
        // choice actions entail rotation & logging, chance action entails board change
        match action {
            Action::Call(_, bet)
            | Action::Blind(_, bet)
            | Action::Raise(_, bet)
            | Action::Shove(_, bet) => {
                seat.bet(bet);
                self.pot += bet;
            }
            _ => (),
        }
        match action {
            Action::Fold(..) => seat.set(BetStatus::Folded),
            Action::Shove(..) => seat.set(BetStatus::Shoved),
            _ => (),
        }
        match action {
            Action::Draw(card) => self.board.push(card),
            _ => {
                self.rotate();
                println!("{action}");
            }
        }
    }
    pub fn begin_hand(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.set(BetStatus::Playing);
            seat.clear();
        }
        self.pot = 0;
        self.counts = 0;
        self.board.cards.clear();
        self.board.street = Street::Pref;
        self.dealer = self.after(self.dealer);
        self.action = self.dealer;
        self.rotate();
    }
    pub fn begin_street(&mut self) {
        self.counts = 0;
        self.action = match self.board.street {
            Street::Pref => self.after(self.after(self.dealer)),
            _ => self.dealer,
        };
        self.rotate();
    }
    pub fn end_street(&mut self) {
        for seat in self.seats.iter_mut() {
            seat.clear();
        }
        self.board.street = match self.board.street {
            Street::Pref => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::Rive,
            Street::Rive => Street::Show,
            Street::Show => unreachable!(),
        }
    }
    fn rotate(&mut self) {
        'left: loop {
            if !self.has_more_players() {
                return;
            }
            self.counts += 1;
            self.action = self.after(self.action);
            match self.seat_up_next().status() {
                BetStatus::Playing => return,
                BetStatus::Folded | BetStatus::Shoved => continue 'left,
            }
        }
    }
    fn rewind(&mut self) {
        'right: loop {
            self.counts -= 1;
            self.action = self.before(self.action);
            match self.seat_up_next().status() {
                BetStatus::Playing => return,
                BetStatus::Folded | BetStatus::Shoved => continue 'right,
            }
        }
    }
    pub fn prune(&mut self) {
        self.seats.retain(|s| s.stack() > 0);
        for (i, seat) in self.seats.iter_mut().enumerate() {
            seat.assign(i);
        }
    }
    pub fn sit_down(&mut self, stack: u32) {
        let position = self.seats.len();
        let seat = Seat::new(stack, position);
        self.seats.push(seat);
    }
    pub fn stand_up(&mut self, position: usize) {
        self.seats.remove(position);
        for (i, seat) in self.seats.iter_mut().enumerate() {
            seat.assign(i);
        }
    }
}
