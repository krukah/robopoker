#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BetStatus {
    Betting,
    Shoved,
    Folded,
}

#[derive(Debug, Clone)]
pub struct Seat {
    pub sunk: u32,
    pub stack: u32,
    pub status: BetStatus,
}

impl Seat {
    pub fn new() -> Seat {
        todo!()
    }

    pub fn bet(&mut self, amount: u32) {
        self.sunk += amount;
        self.stack -= amount;
    }
}
