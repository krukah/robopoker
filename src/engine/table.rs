use super::seat::Seat;

#[derive(Debug, Clone)]
pub struct Table {
    pub seats: Vec<Seat>,
    pub position: usize,
    pub dealer: usize,
    counter: usize,
}

impl Table {
    pub fn new() -> Table {
        todo!()
    }

    pub fn next_hand(&mut self) {
        todo!()
    }

    pub fn next_street(&mut self) {
        todo!()
    }

    pub fn next_player(&mut self) {
        todo!()
    }

    fn is_street_complete(&self) -> bool {
        todo!()
    }

    fn has_all_acted(&self) -> bool {
        todo!()
    }

    fn has_all_folded(&self) -> bool {
        todo!()
    }

    fn has_all_bet(&self) -> bool {
        todo!()
    }
}

impl Iterator for Table {
    type Item = Seat;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
