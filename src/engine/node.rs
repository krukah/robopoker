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
        self.has_all_folded() || (self.has_all_acted() && self.has_all_matched())
    }

    pub fn get_seat(&self) -> &Seat {
        self.seats.get(self.pointer).unwrap()
    }

    pub fn after(&self, i: usize) -> usize {
        (i + 1) % self.seats.len()
    }

    fn has_all_acted(&self) -> bool {
        self.counter > self.seats.len()
    }

    fn has_all_matched(&self) -> bool {
        let bet = self.seats.iter().map(|s| s.sunk).max().unwrap_or(0);
        self.seats
            .iter()
            .filter(|s| s.status == BetStatus::Playing)
            .all(|s| s.sunk == bet)
    }

    fn has_all_folded(&self) -> bool {
        1 == self
            .seats
            .iter()
            .filter(|s| s.status != BetStatus::Folded)
            .count()
    }
}

use super::seat::{BetStatus, Seat};
use crate::cards::board::{Board, Street};
