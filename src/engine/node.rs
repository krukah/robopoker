use super::{action::Action, seat::Seat, table::Table};
use crate::cards::board::{Board, Street};

#[derive(Debug, Clone)]
pub struct Node {
    pub pot: u32,
    pub board: Board,
    pub table: Table,
}

impl Node {
    pub fn new(seats: Vec<Seat>) -> Node {
        Node {
            pot: 0,
            board: Board::new(),
            table: Table::new(seats),
        }
    }

    pub fn apply(&mut self, action: Action) {
        todo!()
    }

    pub fn is_terminal(&self) -> bool {
        match self.board.street {
            Street::River => self.table.is_street_complete(),
            _ => self.table.has_all_folded(),
        }
    }
}
