use super::{action::Action, table::Table};
use crate::cards::board::Board;

#[derive(Debug, Clone)]
pub struct Node {
    pub pot: u32,
    pub board: Board,
    pub table: Table,
}

impl Node {
    pub fn new() -> Node {
        todo!()
    }

    pub fn apply(&mut self, action: Action) {
        todo!()
    }
}
