#[derive(Debug, Clone)]
pub struct Node {
    pub board: Board,
    pub seats: Vec<Seat>,
    pub pot: u32,
    pub dealer: usize,
    pub counter: usize,
    pub pointer: usize,
} // this data struct reads like a poem

impl Node {
    pub fn new() -> Self {
        Node {
            board: Board::new(),
            seats: Vec::with_capacity(10),
            pot: 0,
            dealer: 0,
            counter: 0,
            pointer: 1,
        }
    }

    pub fn is_end_of_hand(&self) -> bool {
        self.has_all_folded() || (self.is_end_of_street() && self.board.street == Street::River)
    }
    pub fn is_end_of_street(&self) -> bool {
        self.has_all_folded() || (self.has_all_decided() && self.has_all_matched())
    }
    pub fn get_seat(&self) -> &Seat {
        self.seats.get(self.pointer).unwrap()
    }
    pub fn get_table_stack(&self) -> u32 {
        let mut stacks: Vec<u32> = self.seats.iter().map(|s| s.stack).collect();
        stacks.sort();
        stacks.pop().unwrap_or(0);
        stacks.pop().unwrap_or(0)
    }
    pub fn get_table_stuck(&self) -> u32 {
        self.seats.iter().map(|s| s.stuck).max().unwrap()
    }

    pub fn after(&self, i: usize) -> usize {
        (i + 1) % self.seats.len()
    }

    fn has_all_decided(&self) -> bool {
        self.counter >= self.seats.len()
    }

    fn has_all_matched(&self) -> bool {
        let bet = self.get_table_stuck();
        self.seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .all(|s| s.stuck == bet)
    }

    fn has_all_folded(&self) -> bool {
        1 == self
            .seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .count()
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

use super::seat::{BetStatus, Seat};
use crate::cards::board::{Board, Street};
use std::fmt::{Display, Formatter, Result};
