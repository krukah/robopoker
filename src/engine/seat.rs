#[derive(Debug, Clone)]
pub struct Seat {
    pub sunk: u32,
    pub stack: u32,
    pub status: SeatStatus,
}

#[derive(Debug, Clone)]
pub enum SeatStatus {
    Alive,
    Shoved,
    Folded,
}
