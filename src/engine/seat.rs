#[derive(Debug, Clone)]
pub struct Seat {
    pub sunk: u32,
    pub stack: u32,
    pub status: BetStatus,
    pub id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BetStatus {
    Playing,
    Shoved,
    Folded,
}

impl Seat {
    pub fn new(stack: u32, position: usize) -> Seat {
        Seat {
            id: position,
            stack,
            sunk: 0,
            status: BetStatus::Playing,
        }
    }
}
