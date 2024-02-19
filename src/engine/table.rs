use super::seat::Seat;

pub struct Table {
    pub seats: Vec<Seat>,
    pub dealer: usize,
    pub index: usize,
}

impl Table {
    fn is_street_complete(&self) -> bool {
        todo!()
    }

    fn is_last_standing(&self) -> bool {
        todo!()
    }
}
