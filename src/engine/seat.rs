use std::fmt::Display;

use crate::cards::hole::Hole;

#[derive(Debug, Clone)]
pub struct Seat {
    pub hole: Hole,
    pub stuck: u32,
    pub stack: u32,
    pub status: BetStatus,
    pub id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BetStatus {
    Playing,
    Shoved,
    Folded,
}

impl Seat {
    pub fn new(stack: u32, position: usize) -> Seat {
        Seat {
            hole: Hole::new(),
            id: position,
            stack,
            stuck: 0,
            status: BetStatus::Playing,
        }
    }
}
impl Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\nSeat {}   Stack {}   {:?}",
            self.id, self.stack, self.status
        )
    }
}
