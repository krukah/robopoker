use super::seat::{Seat, SeatStatus};

#[derive(Debug, Clone)]
pub struct Table {
    counter: usize,
    pub index: usize,
    pub dealer: usize,
    pub seats: Vec<Seat>,
}

impl Table {
    pub fn new(seats: Vec<Seat>) -> Table {
        Table {
            seats,
            index: 0,
            dealer: 0,
            counter: 0,
        }
    }

    pub fn next_hand(&mut self) {
        self.counter = 0;
        self.dealer = self.utg();
        self.index = self.utg();
    }

    pub fn next_street(&mut self) {
        self.counter = 0;
        self.index = self.utg();
    }

    pub fn next_player(&mut self) {
        self.counter += 1;
        todo!()
    }

    pub fn is_street_complete(&self) -> bool {
        self.has_all_folded() || (self.has_all_acted() && self.has_all_bet())
    }

    pub fn has_all_folded(&self) -> bool {
        1 == self
            .seats
            .iter()
            .filter(|s| s.status != SeatStatus::Folded)
            .count()
    }

    pub fn has_all_bet(&self) -> bool {
        let bet = self.seats.iter().map(|s| s.sunk).max().unwrap_or(0);
        self.seats
            .iter()
            .filter(|s| s.status == SeatStatus::Alive)
            .all(|s| s.sunk == bet)
    }

    pub fn has_all_acted(&self) -> bool {
        self.counter
            >= self
                .seats
                .iter()
                .filter(|s| s.status == SeatStatus::Alive)
                .count()
    }

    fn utg(&self) -> usize {
        // check for fenceposts
        (self.dealer + 1) % self.seats.len()
    }
}

impl Iterator for Table {
    type Item = Seat;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
