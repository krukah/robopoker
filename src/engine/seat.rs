pub struct Seat {
    pub sunk: u32,
    pub stack: u32,
    pub status: SeatStatus,
}

pub enum SeatStatus {
    Alive,
    Shoved,
    Folded,
}
