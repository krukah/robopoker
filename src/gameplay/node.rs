// Node represents the memoryless state of the game in between actions. it records both public and private data structs, and is responsible for managing the rotation of players, the pot, and the board. it's immutable methods reveal pure functions representing the rules of how the game may proceed.
#[derive(Debug, Clone)]
pub struct Node {
    pub pot: u32,
    pub dealer: usize,
    pub counter: usize,
    pub pointer: usize,
    pub board: Board,
    pub seats: Vec<Seat>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            seats: Vec::with_capacity(10),
            board: Board::new(),
            pot: 0,
            dealer: 0,
            counter: 0,
            pointer: 0,
        }
    }

    pub fn has_more_hands(&self) -> bool {
        self.seats.iter().filter(|s| s.stack > 2).count() > 1
    }
    pub fn has_more_streets(&self) -> bool {
        !(self.are_all_folded() || (self.board.street == Street::Showdown))
    }
    pub fn has_more_players(&self) -> bool {
        !(self.are_all_folded() || self.are_all_called() || self.are_all_shoved())
    }

    pub fn next(&self) -> &Seat {
        self.seats.get(self.pointer).unwrap()
    }
    pub fn seat(&self, index: usize) -> &Seat {
        self.seats.iter().find(|s| s.position == index).unwrap()
    }
    pub fn seat_mut(&mut self, index: usize) -> &mut Seat {
        self.seats.iter_mut().find(|s| s.position == index).unwrap()
    }
    pub fn after(&self, index: usize) -> usize {
        (index + 1) % self.seats.len()
    }

    pub fn table_stack(&self) -> u32 {
        let mut totals = self
            .seats
            .iter()
            .map(|s| s.stack + s.stake)
            .collect::<Vec<u32>>();
        totals.sort();
        totals.pop().unwrap_or(0);
        totals.pop().unwrap_or(0)
    }
    pub fn table_stake(&self) -> u32 {
        self.seats.iter().map(|s| s.stake).max().unwrap()
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
        let stakes = self.table_stake();
        let is_first_decision = self.counter == 0;
        let is_one_playing = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .count()
            == 1;
        let has_no_decision = is_first_decision && is_one_playing;
        let has_all_decided = self.counter > self.seats.len();
        let has_all_matched = self
            .seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .all(|s| s.stake == stakes);
        (has_all_decided || has_no_decision) && has_all_matched
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Pot:   {}\n", self.pot)?;
        write!(f, "Board: {}", self.board)?;
        for seat in &self.seats {
            write!(f, "{}", seat)?;
        }
        write!(f, "")
    }
}

use super::seat::{BetStatus, Seat};
use crate::cards::board::{Board, Street};
use std::fmt::{Display, Formatter, Result};
